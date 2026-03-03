use anyhow::Result;
use azure_identity::AzureCliCredential;
use azure_ml::MachineLearningServicesClient;

use crate::cli::SubmitArgs;
use crate::code::upload_folder_to_generated_location;

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

    let current_dir = std::env::current_dir()?;
    let source_folder = args.source.as_ref().unwrap_or(&current_dir);

    let code_version =
        upload_folder_to_generated_location(&ml_client, resource_group, workspace, source_folder)
            .await?;

    Ok(())
}
