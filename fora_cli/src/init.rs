use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use azure_identity::AzureCliCredential;
use azure_ml::discovery::{DiscoveryClient, Subscription};
use azure_ml::models::Workspace;
use azure_ml::MachineLearningServicesClient;
use fora_tui::config::{AppConfig, UiConfig, WorkspaceConfig};
use futures::StreamExt;

/// Collected info about a discovered workspace, ready for config generation.
#[derive(Clone, PartialEq, Eq)]
struct DiscoveredWorkspace {
    subscription_id: String,
    resource_group: String,
    name: String,
    location: String,
}

/// Parses the resource group from a full Azure resource ID.
/// Expected format: /subscriptions/{sub}/resourceGroups/{rg}/providers/...
fn parse_resource_group(id: &str) -> Option<String> {
    let parts: Vec<&str> = id.split('/').collect();
    parts
        .iter()
        .position(|&p| p.eq_ignore_ascii_case("resourceGroups"))
        .and_then(|i| parts.get(i + 1))
        .map(|s| s.to_string())
}

/// Converts a generated `Workspace` into a `DiscoveredWorkspace`.
fn to_discovered(workspace: &Workspace, subscription_id: &str) -> Option<DiscoveredWorkspace> {
    let id = workspace.id.as_deref()?;
    let resource_group = parse_resource_group(id)?;
    let name = workspace.name.clone().unwrap_or_default();
    let location = workspace.location.clone().unwrap_or_default();
    Some(DiscoveredWorkspace {
        subscription_id: subscription_id.to_string(),
        resource_group,
        name,
        location,
    })
}

/// Fetches all ML workspaces for a subscription using the generated client.
async fn fetch_workspaces_for_subscription(
    credential: Arc<dyn azure_core::credentials::TokenCredential>,
    subscription: &Subscription,
) -> Result<Vec<DiscoveredWorkspace>> {
    let ml_client = MachineLearningServicesClient::new(
        "https://management.azure.com",
        credential,
        subscription.subscription_id.clone(),
        None,
    )
    .map_err(|e| anyhow::anyhow!("{}", e))
    .context("Failed to create ML client")?;

    let ws_client = ml_client.get_machine_learning_services_workspaces_client();
    let mut pager = ws_client
        .list_by_subscription(None)
        .map_err(|e| anyhow::anyhow!("{}", e))
        .context("Failed to list workspaces")?;

    let mut workspaces = Vec::new();
    while let Some(result) = pager.next().await {
        let workspace: Workspace = result
            .map_err(|e| anyhow::anyhow!("{}", e))
            .context("Failed to fetch workspace page")?;
        if let Some(discovered) = to_discovered(&workspace, &subscription.subscription_id) {
            workspaces.push(discovered);
        }
    }
    Ok(workspaces)
}

/// Runs the interactive config creation wizard.
pub async fn run_init_wizard() -> Result<()> {
    cliclack::intro("fora init")?;

    let credential: Arc<dyn azure_core::credentials::TokenCredential> = AzureCliCredential::new(
        None,
    )
    .context("Failed to create Azure credential. Make sure you are logged in with `az login`.")?;

    let client = DiscoveryClient::new(credential.clone());

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
        let mut prompt =
            cliclack::multiselect("Select subscriptions to use\n  (space to select, enter to confirm)");
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

    // Fetch ML workspaces across all resource groups
    let spinner = cliclack::spinner();
    spinner.start("Fetching ML workspaces...");
    let futures: Vec<_> = selected_subs
        .iter()
        .map(|sub| fetch_workspaces_for_subscription(credential.clone(), sub))
        .collect();
    let results = futures::future::join_all(futures).await;

    let mut ws_entries: Vec<DiscoveredWorkspace> = Vec::new();
    for (result, sub) in results.into_iter().zip(&selected_subs) {
        match result {
            Ok(workspaces) => ws_entries.extend(workspaces),
            Err(e) => {
                cliclack::log::warning(format!(
                    "Failed to list workspaces in {}: {}",
                    sub.display_name, e
                ))?;
            }
        }
    }
    spinner.stop(format!("Found {} ML workspace(s)", ws_entries.len()));

    if ws_entries.is_empty() {
        cliclack::outro_cancel("No ML workspaces found in selected subscriptions.")?;
        return Ok(());
    }

    // Select workspaces
    let selected_workspaces: Vec<DiscoveredWorkspace> = {
        let mut prompt =
            cliclack::multiselect("Select ML workspaces to add\n  (space to select, enter to confirm)");
        for entry in &ws_entries {
            let hint = format!("{} · {}", entry.location, entry.resource_group);
            prompt = prompt.item(entry.clone(), &entry.name, hint);
        }
        if ws_entries.len() == 1 {
            prompt = prompt.initial_values(vec![ws_entries[0].clone()]);
        }
        prompt
            .interact()
            .context("Workspace selection cancelled")?
    };

    // Select default workspace if more than one
    let default_workspace = if selected_workspaces.len() > 1 {
        let mut prompt = cliclack::select("Select default workspace");
        for entry in &selected_workspaces {
            let hint = format!("{} · {}", entry.location, entry.resource_group);
            prompt = prompt.item(entry.name.clone(), &entry.name, hint);
        }
        Some(
            prompt
                .interact()
                .context("Default workspace selection cancelled")?,
        )
    } else {
        None
    };

    // Select timezone
    let tz_names: Vec<String> = chrono_tz::TZ_VARIANTS
        .iter()
        .map(|tz| tz.name().to_string())
        .collect();
    let timezone: String = cliclack::input("Enter your timezone")
        .placeholder("UTC")
        .default_input("UTC")
        .autocomplete(tz_names)
        .validate(|input: &String| {
            if input.parse::<chrono_tz::Tz>().is_ok() {
                Ok(())
            } else {
                Err(
                    "Invalid timezone. Use an IANA name like Europe/London or America/New_York."
                        .to_string(),
                )
            }
        })
        .interact()
        .context("Timezone input cancelled")?;
    let timezone = if timezone == "UTC" {
        None
    } else {
        Some(timezone)
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
            name: entry.name.clone(),
            subscription_id: entry.subscription_id.clone(),
            resource_group: entry.resource_group.clone(),
            workspace_name: entry.name.clone(),
            region: entry.location.clone(),
        })
        .collect();

    let config = AppConfig {
        ui: UiConfig {
            timezone,
            ..Default::default()
        },
        default_workspace,
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
