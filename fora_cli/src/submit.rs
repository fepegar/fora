use anyhow::anyhow;
use anyhow::Result;
use azure_identity::AzureCliCredential;
use azure_ml::MachineLearningServicesClient;
use futures::future::join_all;

use crate::cli::EnvironmentType;
use crate::cli::SubmitArgs;
use crate::code::upload_folder_to_generated_location;
use crate::env::Environment;
use crate::env::UvEnv;

pub async fn submit_to_azure(args: &SubmitArgs) -> Result<()> {
    // Upload code
    let resource_group = &args.resource_group;
    let workspace = &args.workspace;
    let subscription = &args.subscription;

    // Create Azure credential
    let credential = AzureCliCredential::new(None)?;

    // Create the Machine Learning Services client
    let ml_client = MachineLearningServicesClient::new(
        "https://management.azure.com",
        credential,
        subscription.clone(),
        None,
    )?;

    // pub auto_rebuild: bool,
    // pub base_docker_image: String,
    // pub project_dir: PathBuf,
    // pub uv_extras: Vec<String>,
    // pub uv_groups: Vec<String>,

    let default_docker_image = "mcr.microsoft.com/azureml/openmpi5.0-cuda12.6-ubuntu24.04";
    let current_dir = std::env::current_dir()?;
    let source_folder = args.source.as_ref().unwrap_or(&current_dir);

    let env = match args.env_args.env_type {
        EnvironmentType::Uv => {
            let env = UvEnv {
                auto_rebuild: args.env_args.auto_rebuild,
                base_docker_image: args
                    .env_args
                    .base_docker_image
                    .clone()
                    .unwrap_or(default_docker_image.to_string()),
                project_dir: args
                    .uv_args
                    .project_dir
                    .clone()
                    .unwrap_or(source_folder.clone()),
                uv_extras: args.uv_args.uv_extra.clone(),
                uv_groups: args.uv_args.uv_group.clone(),
            };
            Ok(env)
        }
        _ => Err(anyhow!("Not implemented")),
    }?;

    let code_version_future =
        upload_folder_to_generated_location(&ml_client, resource_group, workspace, source_folder);

    let env_future = env.get_or_create_environment(&ml_client, resource_group, workspace);

    let (_code_version_result, _env_result) = futures::join!(code_version_future, env_future);

    Ok(())
}
