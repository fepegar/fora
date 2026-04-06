# Setup

Fora is configured via a TOML file. You must set up at least one workspace before using the TUI or submitting jobs.

## Config file location

Fora looks for configuration in two places, in order of priority:

1. **Local** — `./fora.toml` in the current working directory
2. **Global** — `~/.config/fora.toml`

If both exist, the local config takes priority.

## Authentication

Fora authenticates with Azure via the Azure CLI. Make sure you are logged in before running Fora:

```bash
az login
```

## Config file format

Here is a complete example configuration:

```toml
# Your Azure ML username — used to filter "My Recent Jobs" in the TUI.
# Must match the mlflow.user tag on your jobs.
username = "your-username"

[ui]
# Show the keyboard shortcut help bar at the bottom of the TUI.
show_help_bar = true
# Auto-refresh interval in seconds.
refresh_interval_secs = 30

# Define one or more Azure ML workspaces.
[[workspaces]]
name = "prod"
subscription_id = "00000000-0000-0000-0000-000000000000"
resource_group = "my-resource-group"
workspace_name = "my-ml-workspace"
# Azure region — used for the MLflow API endpoint.
region = "eastus2"

[[workspaces]]
name = "dev"
subscription_id = "11111111-1111-1111-1111-111111111111"
resource_group = "dev-rg"
workspace_name = "dev-workspace"
region = "westus2"

# Optional: customise which columns are visible in each tab.
# Listed columns appear in order; unlisted columns are hidden.
[columns]
jobs = ["name", "status", "experiment", "created"]
compute = ["name", "type", "state", "vm_size"]
```

## Fields reference

### Top-level

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `username` | string | Yes | Your username for filtering recent jobs. Must match the `mlflow.user` tag. |

### `[ui]`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `show_help_bar` | bool | `true` | Show the keyboard shortcut help bar at the bottom of the TUI. |
| `refresh_interval_secs` | integer | `30` | Auto-refresh interval in seconds. |

### `[[workspaces]]`

Each workspace entry defines an Azure ML workspace to connect to:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Display name for the workspace (shown in the workspace picker). |
| `subscription_id` | string | Yes | Azure subscription ID. |
| `resource_group` | string | Yes | Azure resource group name. |
| `workspace_name` | string | Yes | Azure ML workspace name. |
| `region` | string | Yes | Azure region (e.g., `eastus2`). Used for the MLflow API endpoint. |

### `[columns]`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `jobs` | list of strings | All columns | Columns to display in the Jobs tab. |
| `compute` | list of strings | All columns | Columns to display in the Compute tab. |

Column configuration can also be changed interactively from within the TUI using the column picker.

## Minimal config

The smallest useful config file:

```toml
username = "your-username"

[[workspaces]]
name = "my-workspace"
subscription_id = "00000000-0000-0000-0000-000000000000"
resource_group = "my-rg"
workspace_name = "my-ml-workspace"
region = "eastus2"
```
