use anyhow::{Context, Result};
use azure_identity::AzureCliCredential;
use azure_ml::MachineLearningServicesClient;
use clap::Parser;

#[derive(Parser)]
#[command(name = "fora_cli")]
#[command(about = "A CLI tool for Azure Machine Learning operations")]
struct Args {
    /// Azure subscription ID
    #[arg(short, long)]
    subscription_id: String,

    /// Resource group name
    #[arg(short, long)]
    resource_group: String,

    /// Workspace name
    #[arg(short, long)]
    workspace_name: String,

    /// Job ID to get details for
    #[arg(short, long)]
    job_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Create Azure credential
    let credential = AzureCliCredential::new(None)?;

    // Create the Machine Learning Services client
    let client = MachineLearningServicesClient::new(
        "https://management.azure.com",
        credential,
        args.subscription_id,
        None,
    )
    .context("Failed to create Azure ML client")?;

    // Get the jobs client
    let jobs_client = client.get_machine_learning_services_jobs_client();

    // Get specific job details
    let response = jobs_client
        .get(
            &args.resource_group,
            &args.workspace_name,
            &args.job_id,
            None,
        )
        .await
        .context("Failed to get job details")?;

    let job = response.body().json::<azure_ml::models::JobBase>()?;

    // Print job information
    println!("Job Details for ID '{}':", args.job_id);
    println!("Name: {:?}", job.name);
    println!("ID: {:?}", job.id);
    println!("Type: {:?}", job.type_prop);

    if let Some(properties) = job.properties {
        println!("Properties: {:#?}", properties);
    }

    if let Some(system_data) = job.system_data {
        println!("System Data: {:#?}", system_data);
    }

    Ok(())
}
