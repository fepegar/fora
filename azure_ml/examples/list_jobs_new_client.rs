//! Example: List jobs using the new AzureMLClient
//!
//! This example demonstrates how to use the new AzureMLClient to list jobs
//! in an Azure ML workspace with various query parameters.
//!
//! To run this example:
//! ```bash
//! cargo run --example list_jobs_new_client
//! ```
//!
//! Make sure you have Azure credentials configured and set the required environment variables:
//! - AZURE_SUBSCRIPTION_ID
//! - AZURE_RESOURCE_GROUP
//! - AZURE_ML_WORKSPACE

use azure_core::http::Pipeline;
use azure_core::ClientOptions;
use azure_identity::DefaultAzureCredential;
use azure_ml::{AzureMLClient, ListViewType};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::init();

    // Get configuration from environment variables
    let subscription_id = env::var("AZURE_SUBSCRIPTION_ID")
        .expect("AZURE_SUBSCRIPTION_ID environment variable is required");
    let resource_group_name = env::var("AZURE_RESOURCE_GROUP")
        .expect("AZURE_RESOURCE_GROUP environment variable is required");
    let workspace_name = env::var("AZURE_ML_WORKSPACE")
        .expect("AZURE_ML_WORKSPACE environment variable is required");

    // Create Azure credentials
    let credential = DefaultAzureCredential::default();

    // Create pipeline with credentials
    let mut pipeline = Pipeline::new(
        option_env!("CARGO_PKG_NAME").unwrap_or("azure-ml-client"),
        option_env!("CARGO_PKG_VERSION").unwrap_or("0.1.0"),
        ClientOptions::default(),
        Vec::new(),
        Vec::new(),
    );

    // Add authentication policy to pipeline
    pipeline.push(
        azure_core::policies::AuthorizationPolicy::new(
            Arc::new(credential),
            "https://management.azure.com/",
        ),
        azure_core::policies::PolicyPosition::PerCall,
    );

    // Create the Azure ML client
    let client = AzureMLClient::from_pipeline(
        "https://management.azure.com".to_string(),
        subscription_id.clone(),
        pipeline,
        Some("2025-09-01".to_string()),
    );

    println!("🚀 Starting Azure ML jobs listing example...");
    println!("   Subscription ID: {}", subscription_id);
    println!("   Resource Group: {}", resource_group_name);
    println!("   Workspace: {}", workspace_name);

    // Example 1: List all jobs with basic parameters
    println!("\n📋 Example 1: List all jobs");
    match client
        .list_jobs(
            &resource_group_name,
            &workspace_name,
            None,                    // skip
            None,                    // job_type
            None,                    // tag
            Some(ListViewType::All), // list_view_type
            None,                    // properties
        )
        .await
    {
        Ok(jobs_response) => {
            if let Some(jobs) = &jobs_response.value {
                println!("✅ Found {} job(s)", jobs.len());
                for (i, job) in jobs.iter().take(3).enumerate() {
                    println!("   {}. Job ID: {:?}", i + 1, job.name);
                    if let Some(properties) = &job.properties {
                        println!("      Status: {:?}", properties.status);
                        println!("      Job Type: {:?}", properties.job_type);
                    }
                }
                if jobs.len() > 3 {
                    println!("      ... and {} more jobs", jobs.len() - 3);
                }
            } else {
                println!("📭 No jobs found");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to list jobs: {}", e);
        }
    }

    // Example 2: List jobs with pagination
    println!("\n📋 Example 2: List jobs with pagination (skip first 2)");
    match client
        .list_jobs(
            &resource_group_name,
            &workspace_name,
            Some(2),                        // skip first 2 jobs
            None,                           // job_type
            None,                           // tag
            Some(ListViewType::ActiveOnly), // only active jobs
            None,                           // properties
        )
        .await
    {
        Ok(jobs_response) => {
            if let Some(jobs) = &jobs_response.value {
                println!("✅ Found {} job(s) (after skipping 2)", jobs.len());
                for (i, job) in jobs.iter().take(2).enumerate() {
                    println!("   {}. Job ID: {:?}", i + 3, job.name); // +3 because we skipped 2
                    if let Some(properties) = &job.properties {
                        println!("      Status: {:?}", properties.status);
                    }
                }
            } else {
                println!("📭 No more jobs found");
            }

            if let Some(next_link) = &jobs_response.next_link {
                println!("🔗 Next page available: {}", next_link);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to list jobs with pagination: {}", e);
        }
    }

    // Example 3: Filter by job type
    println!("\n📋 Example 3: Filter jobs by type (Command jobs)");
    match client
        .list_jobs(
            &resource_group_name,
            &workspace_name,
            None,                           // skip
            Some("Command"),                // job_type filter
            None,                           // tag
            Some(ListViewType::ActiveOnly), // list_view_type
            None,                           // properties
        )
        .await
    {
        Ok(jobs_response) => {
            if let Some(jobs) = &jobs_response.value {
                println!("✅ Found {} Command job(s)", jobs.len());
                for (i, job) in jobs.iter().take(3).enumerate() {
                    println!("   {}. Job ID: {:?}", i + 1, job.name);
                    if let Some(properties) = &job.properties {
                        println!("      Job Type: {:?}", properties.job_type);
                        println!("      Status: {:?}", properties.status);
                    }
                }
            } else {
                println!("📭 No Command jobs found");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to filter jobs by type: {}", e);
        }
    }

    // Example 4: Filter by tag
    println!("\n📋 Example 4: Filter jobs by tag");
    match client
        .list_jobs(
            &resource_group_name,
            &workspace_name,
            None,                           // skip
            None,                           // job_type
            Some("environment=production"), // tag filter
            Some(ListViewType::All),        // list_view_type
            None,                           // properties
        )
        .await
    {
        Ok(jobs_response) => {
            if let Some(jobs) = &jobs_response.value {
                println!("✅ Found {} job(s) with production tag", jobs.len());
                for (i, job) in jobs.iter().take(2).enumerate() {
                    println!("   {}. Job ID: {:?}", i + 1, job.name);
                }
            } else {
                println!("📭 No jobs found with the specified tag");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to filter jobs by tag: {}", e);
        }
    }

    // Example 5: Request specific properties
    println!("\n📋 Example 5: List jobs with specific properties");
    match client
        .list_jobs(
            &resource_group_name,
            &workspace_name,
            None,                           // skip
            None,                           // job_type
            None,                           // tag
            Some(ListViewType::ActiveOnly), // list_view_type
            Some("status,createdDateTime"), // only return these properties
        )
        .await
    {
        Ok(jobs_response) => {
            if let Some(jobs) = &jobs_response.value {
                println!("✅ Found {} job(s) with limited properties", jobs.len());
                for (i, job) in jobs.iter().take(3).enumerate() {
                    println!("   {}. Job ID: {:?}", i + 1, job.name);
                    if let Some(properties) = &job.properties {
                        println!("      Status: {:?}", properties.status);
                        // Note: createdDateTime might not be available in the limited property set
                    }
                }
            } else {
                println!("📭 No jobs found");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to list jobs with specific properties: {}", e);
        }
    }

    println!("\n🎉 All examples completed!");
    println!("\n💡 Tips:");
    println!("   - Use ListViewType::ActiveOnly to see only active jobs");
    println!("   - Use ListViewType::ArchivedOnly to see archived jobs");
    println!("   - Use ListViewType::All to see both active and archived jobs");
    println!("   - Use the skip parameter for pagination");
    println!("   - Filter by jobType: Command, Pipeline, AutoML, Sweep, etc.");
    println!("   - Use tag filters to find jobs with specific metadata");
    println!("   - Use properties parameter to limit returned data and improve performance");

    Ok(())
}
