//! Example: List jobs in an Azure ML workspace
//!
//! This example demonstrates how to use the AzureMLClient to list jobs
//! in an Azure ML workspace.
//!
//! To run this example:
//! ```bash
//! cargo run --example list_jobs
//! ```
//!
//! Make sure you have Azure credentials configured (e.g., via Azure CLI)
//! and set the required environment variables:
//! - AZURE_SUBSCRIPTION_ID
//! - AZURE_RESOURCE_GROUP
//! - AZURE_ML_WORKSPACE

use azure_ml::{AzureMLClient, AzureMLConfig};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable logging
    tracing_subscriber::init();

    // Get configuration from environment variables
    let subscription_id = env::var("AZURE_SUBSCRIPTION_ID")
        .expect("AZURE_SUBSCRIPTION_ID environment variable is required");
    let resource_group_name = env::var("AZURE_RESOURCE_GROUP")
        .expect("AZURE_RESOURCE_GROUP environment variable is required");
    let workspace_name = env::var("AZURE_ML_WORKSPACE")
        .expect("AZURE_ML_WORKSPACE environment variable is required");

    // Create the Azure ML client configuration
    let config = AzureMLConfig {
        subscription_id,
        resource_group_name,
        workspace_name,
        ..Default::default()
    };

    println!("Creating Azure ML client...");
    let client = AzureMLClient::new(config)?;

    println!("Fetching workspace information...");
    match client.get_workspace().await {
        Ok(workspace) => {
            println!("✅ Connected to workspace successfully!");
            if let Some(name) = workspace.get("name") {
                println!("   Workspace: {}", name);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to workspace: {}", e);
            return Err(e.into());
        }
    }

    println!("\nListing jobs...");
    match client.list_jobs().await {
        Ok(jobs_response) => {
            println!("✅ Successfully retrieved jobs");

            // Pretty print the response
            if let Some(value) = jobs_response.get("value") {
                if let Some(jobs_array) = value.as_array() {
                    println!("Found {} job(s):", jobs_array.len());

                    for (i, job) in jobs_array.iter().enumerate() {
                        println!("\n--- Job {} ---", i + 1);

                        if let Some(name) = job.get("name") {
                            println!("Name: {}", name);
                        }

                        if let Some(properties) = job.get("properties") {
                            if let Some(status) = properties.get("status") {
                                println!("Status: {}", status);
                            }
                            if let Some(job_type) = properties.get("jobType") {
                                println!("Type: {}", job_type);
                            }
                            if let Some(created_time) = properties.get("createdDateTime") {
                                println!("Created: {}", created_time);
                            }
                        }
                    }
                } else {
                    println!("No jobs found in the response");
                }
            } else {
                println!("Unexpected response format");
                println!(
                    "Full response: {}",
                    serde_json::to_string_pretty(&jobs_response)?
                );
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to list jobs: {}", e);
            return Err(e.into());
        }
    }

    println!("\n🎉 Example completed successfully!");
    Ok(())
}
