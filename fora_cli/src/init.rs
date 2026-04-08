use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use azure_identity::AzureCliCredential;
use azure_ml::discovery::{DiscoveryClient, MlWorkspace, ResourceGroup, Subscription};
use fora_tui::config::{AppConfig, WorkspaceConfig};

/// Collected info about a discovered workspace, ready for config generation.
#[derive(Clone, PartialEq, Eq)]
struct DiscoveredWorkspace {
    subscription_id: String,
    resource_group: String,
    workspace: MlWorkspace,
}

/// Runs the interactive config creation wizard.
pub async fn run_init_wizard() -> Result<()> {
    cliclack::intro("fora init")?;

    let credential: Arc<dyn azure_core::credentials::TokenCredential> = AzureCliCredential::new(
        None,
    )
    .context("Failed to create Azure credential. Make sure you are logged in with `az login`.")?;

    let client = DiscoveryClient::new(credential);

    // Fetch user profile
    let spinner = cliclack::spinner();
    spinner.start("Fetching your Azure profile...");
    match client.get_current_user().await {
        Ok(profile) => {
            spinner.stop(format!("Welcome, {}!", profile.name()));
        }
        Err(e) => {
            spinner.stop(format!("Could not fetch profile: {}", e));
        }
    }

    // Fetch subscriptions
    let spinner = cliclack::spinner();
    spinner.start("Fetching subscriptions...");
    let subscriptions = client
        .list_subscriptions()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
        .context("Failed to list subscriptions")?;
    spinner.stop(format!("Found {} subscription(s)", subscriptions.len()));

    if subscriptions.is_empty() {
        cliclack::outro_cancel("No subscriptions found. Check your Azure login.")?;
        return Ok(());
    }

    // Select subscriptions
    let selected_subs: Vec<Subscription> = {
        let mut prompt = cliclack::multiselect("Select subscriptions to use");
        for sub in &subscriptions {
            prompt = prompt.item(sub.clone(), &sub.display_name, &sub.subscription_id);
        }
        if subscriptions.len() == 1 {
            prompt = prompt.initial_values(vec![subscriptions[0].clone()]);
        }
        prompt
            .interact()
            .context("Subscription selection cancelled")?
    };

    // Fetch resource groups for selected subscriptions
    let spinner = cliclack::spinner();
    spinner.start("Fetching resource groups...");
    let mut rg_entries: Vec<(ResourceGroup, Subscription)> = Vec::new();
    for sub in &selected_subs {
        match client.list_resource_groups(&sub.subscription_id).await {
            Ok(rgs) => {
                for rg in rgs {
                    rg_entries.push((rg, sub.clone()));
                }
            }
            Err(e) => {
                spinner.stop(format!(
                    "Warning: failed to list resource groups for {}: {}",
                    sub.display_name, e
                ));
                let spinner_new = cliclack::spinner();
                spinner_new.start("Continuing...");
                // Continue to next subscription
            }
        }
    }
    spinner.stop(format!("Found {} resource group(s)", rg_entries.len()));

    if rg_entries.is_empty() {
        cliclack::outro_cancel("No resource groups found in selected subscriptions.")?;
        return Ok(());
    }

    // Select resource groups
    let show_sub_hint = selected_subs.len() > 1;
    let selected_rgs: Vec<(ResourceGroup, Subscription)> = {
        let mut prompt = cliclack::multiselect("Select resource groups");
        for (rg, sub) in &rg_entries {
            let hint = if show_sub_hint {
                sub.display_name.clone()
            } else {
                rg.location.clone()
            };
            prompt = prompt.item((rg.clone(), sub.clone()), &rg.name, hint);
        }
        prompt
            .interact()
            .context("Resource group selection cancelled")?
    };

    // Fetch ML workspaces for selected resource groups
    let spinner = cliclack::spinner();
    spinner.start("Fetching ML workspaces...");
    let mut ws_entries: Vec<DiscoveredWorkspace> = Vec::new();
    for (rg, sub) in &selected_rgs {
        match client
            .list_ml_workspaces(&sub.subscription_id, &rg.name)
            .await
        {
            Ok(workspaces) => {
                for ws in workspaces {
                    ws_entries.push(DiscoveredWorkspace {
                        subscription_id: sub.subscription_id.clone(),
                        resource_group: rg.name.clone(),
                        workspace: ws,
                    });
                }
            }
            Err(e) => {
                cliclack::log::warning(format!(
                    "Failed to list workspaces in {}/{}: {}",
                    sub.display_name, rg.name, e
                ))?;
            }
        }
    }
    spinner.stop(format!("Found {} ML workspace(s)", ws_entries.len()));

    if ws_entries.is_empty() {
        cliclack::outro_cancel("No ML workspaces found in selected resource groups.")?;
        return Ok(());
    }

    // Select workspaces
    let selected_workspaces: Vec<DiscoveredWorkspace> = {
        let mut prompt = cliclack::multiselect("Select ML workspaces to add");
        for entry in &ws_entries {
            let hint = format!("{} · {}", entry.workspace.location, entry.resource_group);
            prompt = prompt.item(entry.clone(), &entry.workspace.name, hint);
        }
        if ws_entries.len() == 1 {
            prompt = prompt.initial_values(vec![ws_entries[0].clone()]);
        }
        prompt.interact().context("Workspace selection cancelled")?
    };

    // Select config file location
    let global_path = AppConfig::global_config_path();
    let config_location: String = cliclack::select("Where should the config file be saved?")
        .item(
            global_path.clone(),
            format!("{} (global)", global_path),
            "recommended",
        )
        .item(
            "./fora.toml".to_string(),
            "./fora.toml (local)",
            "project-specific",
        )
        .interact()
        .context("Config location selection cancelled")?;

    let config_path = PathBuf::from(&config_location);

    // Check if config already exists
    if config_path.exists() {
        let overwrite: bool =
            cliclack::confirm(format!("{} already exists. Overwrite?", config_location))
                .interact()
                .context("Overwrite confirmation cancelled")?;

        if !overwrite {
            cliclack::outro_cancel("Cancelled. Existing config file was not modified.")?;
            return Ok(());
        }
    }

    // Build config
    let workspace_configs: Vec<WorkspaceConfig> = selected_workspaces
        .iter()
        .map(|entry| WorkspaceConfig {
            name: entry.workspace.name.clone(),
            subscription_id: entry.subscription_id.clone(),
            resource_group: entry.resource_group.clone(),
            workspace_name: entry.workspace.name.clone(),
            region: entry.workspace.location.clone(),
        })
        .collect();

    let config = AppConfig {
        ui: Default::default(),
        workspaces: workspace_configs,
        columns: Default::default(),
        config_path: Some(config_path.clone()),
    };

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let toml_str = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, &toml_str)?;

    cliclack::outro(format!(
        "Configuration saved to {}. Run `fora` to start the TUI!",
        config_location
    ))?;

    Ok(())
}
