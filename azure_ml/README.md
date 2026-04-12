# Azure Machine Learning Rust Client

This is an auto-generated Rust client for the Azure Machine Learning Services REST API, generated from TypeSpec definitions using the `@azure-tools/typespec-rust` emitter.

## Overview

This client provides a complete Rust interface to Azure Machine Learning Services, including:

- **Workspaces**: Manage ML workspaces
- **Compute**: Manage compute resources (AKS, AML Compute, Compute Instances, etc.)
- **Datasets**: Manage data containers and versions
- **Models**: Manage model containers and versions
- **Endpoints**: Manage online and batch endpoints
- **Jobs**: Manage training and inference jobs
- **Environments**: Manage ML environments
- **Components**: Manage reusable ML components
- **Datastores**: Manage data storage connections

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
azure_mgmt_machinelearning = { path = "path/to/this/crate" }
azure_core = "0.21"
azure_identity = "0.21"
tokio = { version = "1.0", features = ["full"] }
```

## Authentication

You'll need Azure credentials to use this client. The recommended approach is to use `azure_identity`:

```rust
use azure_identity::DefaultAzureCredential;
use azure_mgmt_machinelearning::MachineLearningServicesClient;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credential = Arc::new(DefaultAzureCredential::default());
    let subscription_id = "your-subscription-id".to_string();
    
    let client = MachineLearningServicesClient::new(
        "https://management.azure.com",
        credential,
        subscription_id,
        None
    )?;
    
    // Use the client...
    
    Ok(())
}
```

## Usage Examples

### List Workspaces

```rust
use azure_mgmt_machinelearning::MachineLearningServicesClient;

async fn list_workspaces(client: &MachineLearningServicesClient, resource_group: &str) {
    let workspaces = client.workspaces()
        .list_by_resource_group(resource_group)
        .await
        .expect("Failed to list workspaces");
    
    for workspace in workspaces.value {
        println!("Workspace: {}", workspace.name);
    }
}
```

### Create a Compute Instance

```rust
use azure_mgmt_machinelearning::models::*;

async fn create_compute_instance(
    client: &MachineLearningServicesClient,
    resource_group: &str,
    workspace_name: &str,
    compute_name: &str
) {
    let compute_instance = ComputeInstance {
        // Configure your compute instance properties
        properties: Some(ComputeInstanceProperties {
            vm_size: Some("STANDARD_DS3_V2".to_string()),
            // ... other properties
        }),
        // ... other fields
    };
    
    let result = client.compute()
        .begin_create_or_update(
            resource_group,
            workspace_name,
            compute_name,
            &compute_instance
        )
        .await
        .expect("Failed to create compute instance");
}
```

### Submit a Training Job

```rust
use azure_mgmt_machinelearning::models::*;

async fn submit_job(
    client: &MachineLearningServicesClient,
    resource_group: &str,
    workspace_name: &str,
    job_name: &str
) {
    let command_job = CommandJob {
        // Configure your job properties
        command: Some("python train.py".to_string()),
        environment_id: Some("azureml:python-sklearn:1".to_string()),
        compute_id: Some("your-compute-target".to_string()),
        // ... other properties
    };
    
    let result = client.jobs()
        .create_or_update(
            resource_group,
            workspace_name,
            job_name,
            &command_job
        )
        .await
        .expect("Failed to submit job");
}
```

## Client Structure

The main `MachineLearningServicesClient` provides access to various service clients:

- `client.workspaces()` - Workspace operations
- `client.compute()` - Compute resource operations  
- `client.jobs()` - Job operations
- `client.models()` - Model operations
- `client.data_containers()` - Data operations
- `client.online_endpoints()` - Online endpoint operations
- `client.batch_endpoints()` - Batch endpoint operations
- `client.datastores()` - Datastore operations
- And many more...

## Error Handling

All operations return `Result` types. Use proper error handling:

```rust
match client.workspaces().get(resource_group, workspace_name).await {
    Ok(workspace) => {
        println!("Found workspace: {}", workspace.name);
    }
    Err(e) => {
        eprintln!("Error getting workspace: {}", e);
    }
}
```

## Current Limitations

⚠️ **Note**: This generated client currently has some compilation issues due to naming conflicts in union types. This is a known limitation of the TypeSpec Rust emitter for complex Azure APIs.

To work around this:
1. You may need to manually resolve naming conflicts in the generated code
2. Consider using specific operation clients rather than the full models
3. The Azure SDK for Rust team is working on improvements to handle these cases better

## Alternative Approaches

If you encounter issues with this generated client, consider:

1. **Using the official Azure SDK for Rust** (if available for ML Services)
2. **Making direct HTTP requests** using libraries like `reqwest` with Azure authentication
3. **Using the Azure CLI** via process calls for simpler operations

## Contributing

This client is auto-generated from TypeSpec definitions. To report issues or contribute:

1. Issues with the generated client: Report to the Azure TypeSpec Rust emitter repository
2. Issues with the API definitions: Report to the Azure REST API specs repository
3. Issues with Azure ML Services: Report to Azure support

## License

This project is licensed under the MIT License - see the file headers for details.
