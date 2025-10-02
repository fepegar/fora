# AzureMLClient.list_jobs() Method

The `list_jobs` method allows you to retrieve jobs from an Azure ML workspace with various filtering and pagination options.

## Method Signature

```rust
pub async fn list_jobs(
    &self,
    resource_group: &str,
    workspace: &str,
    skip: Option<i32>,
    job_type: Option<&str>,
    tag: Option<&str>,
    list_view_type: Option<models::ListViewType>,
    properties: Option<&str>,
) -> AzureResult<models::JobBaseResourceArmPaginatedResult>
```

## Parameters

- **resource_group** (`&str`): The Azure resource group name containing the workspace
- **workspace** (`&str`): The Azure ML workspace name
- **skip** (`Option<i32>`): Number of jobs to skip for pagination (optional)
- **job_type** (`Option<&str>`): Filter jobs by type (e.g., "Command", "Pipeline", "AutoML", "Sweep") (optional)
- **tag** (`Option<&str>`): Filter jobs that have this tag key (optional)
- **list_view_type** (`Option<ListViewType>`): View type for including/excluding archived entities (optional)
- **properties** (`Option<&str>`): Comma-separated list of properties to return (optional)

## ListViewType Options

- `ListViewType::ActiveOnly` - Show only active (non-archived) jobs (default)
- `ListViewType::ArchivedOnly` - Show only archived jobs
- `ListViewType::All` - Show both active and archived jobs

## Return Type

Returns `JobBaseResourceArmPaginatedResult` which contains:
- `value`: Optional vector of `JobBaseResource` objects
- `next_link`: Optional URL for the next page of results

## Usage Examples

### Basic Usage - List All Active Jobs

```rust
let jobs = client.list_jobs(
    "my-resource-group",
    "my-workspace",
    None,                           // skip
    None,                           // job_type
    None,                           // tag
    Some(ListViewType::ActiveOnly), // list_view_type
    None,                           // properties
).await?;

if let Some(jobs) = jobs.value {
    println!("Found {} jobs", jobs.len());
    for job in jobs {
        println!("Job: {}", job.name.unwrap_or_default());
    }
}
```

### Pagination

```rust
// Get first 10 jobs
let page1 = client.list_jobs(
    "my-resource-group",
    "my-workspace",
    None,                           // start from beginning
    None, None,
    Some(ListViewType::ActiveOnly),
    None,
).await?;

// Get next 10 jobs (skip first 10)
let page2 = client.list_jobs(
    "my-resource-group", 
    "my-workspace",
    Some(10),                       // skip first 10
    None, None,
    Some(ListViewType::ActiveOnly),
    None,
).await?;
```

### Filter by Job Type

```rust
// Get only Command jobs
let command_jobs = client.list_jobs(
    "my-resource-group",
    "my-workspace", 
    None,
    Some("Command"),                // filter by job type
    None,
    Some(ListViewType::ActiveOnly),
    None,
).await?;
```

### Filter by Tag

```rust
// Get jobs with specific tag
let tagged_jobs = client.list_jobs(
    "my-resource-group",
    "my-workspace",
    None,
    None,
    Some("environment=production"), // filter by tag
    Some(ListViewType::All),
    None,
).await?;
```

### Limit Returned Properties

```rust
// Only return specific properties for better performance
let jobs = client.list_jobs(
    "my-resource-group",
    "my-workspace",
    None,
    None,
    None,
    Some(ListViewType::ActiveOnly),
    Some("status,createdDateTime,jobType"), // limit properties
).await?;
```

### Include Archived Jobs

```rust
// Get all jobs including archived ones
let all_jobs = client.list_jobs(
    "my-resource-group",
    "my-workspace",
    None,
    None, 
    None,
    Some(ListViewType::All),        // include archived
    None,
).await?;
```

## Common Job Types

- `"Command"` - Command line jobs
- `"Pipeline"` - ML pipelines
- `"AutoML"` - Automated ML jobs
- `"Sweep"` - Hyperparameter tuning jobs
- `"Spark"` - Spark jobs

## Error Handling

The method returns `AzureResult<JobBaseResourceArmPaginatedResult>`. Handle errors appropriately:

```rust
match client.list_jobs(resource_group, workspace, None, None, None, None, None).await {
    Ok(jobs_response) => {
        // Process jobs
        if let Some(jobs) = jobs_response.value {
            println!("Found {} jobs", jobs.len());
        }
    }
    Err(e) => {
        eprintln!("Failed to list jobs: {}", e);
    }
}
```

## API Endpoint

This method calls the Azure ML REST API endpoint:
```
GET https://management.azure.com/subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/Microsoft.MachineLearningServices/workspaces/{workspaceName}/jobs?api-version=2025-09-01
```

## Notes

- The method uses API version `2025-09-01` by default
- Query parameters are properly URL-encoded
- Pagination is handled via the `skip` parameter and `next_link` in the response
- All parameters except `resource_group` and `workspace` are optional
- The `properties` parameter can improve performance by limiting returned data