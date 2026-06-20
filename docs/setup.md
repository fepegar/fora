# Setup

Fora is configured via a TOML file. You must set up at least one workspace before using the TUI.

## Quick start

The easiest way to create a config file is to run the interactive setup wizard:

```bash
fora init
```

This will discover your Azure subscriptions, resource groups, and ML workspaces, then generate a config file. If you run `fora` without a config file, the wizard starts automatically.

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

Your display name is fetched automatically from Microsoft Graph when the TUI starts, so there is no need to configure it manually.

## Config file format

Here is a complete example configuration:

```toml
[ui]
# Show the keyboard shortcut help bar at the bottom of the TUI.
show_help_bar = true
# Auto-refresh interval in seconds.
refresh_interval_secs = 30
# IANA timezone for displaying timestamps (e.g. "Europe/London", "America/New_York").
# Handles daylight saving time automatically. Defaults to UTC if unset.
timezone = "Europe/London"
# How often the Files sub-tab re-checks the currently-previewed file
# (in seconds) while the run is active. Set 0 to disable live tailing.
file_preview_refresh_secs = 3
# syntect theme name used by the Files sub-tab for highlighted previews.
# Any theme bundled with syntect works (e.g. "InspiredGitHub",
# "Solarized (dark)", "base16-eighties.dark").
syntax_theme = "base16-ocean.dark"
# Default directory pre-filled in the download prompt opened by the `s`
# hotkey in the Files sub-tab. `~` is expanded; relative paths resolve
# against the current working directory.
save_dir = "./"

# Workspace shown when launching the TUI (must match a workspace name below).
default_workspace = "prod"

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

### `[ui]`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `show_help_bar` | bool | `true` | Show the keyboard shortcut help bar at the bottom of the TUI. |
| `refresh_interval_secs` | integer | `30` | Auto-refresh interval in seconds. |
| `timezone` | string | `"UTC"` | IANA timezone name for displaying timestamps (e.g. `"Europe/London"`, `"America/New_York"`). Handles daylight saving time automatically. |
| `file_preview_refresh_secs` | integer | `3` | Polling interval used by the Files sub-tab to re-check the previewed file while the run is active. Set `0` to disable live tailing. |
| `syntax_theme` | string | `"base16-ocean.dark"` | syntect theme used for highlighted file previews. Any theme bundled with `syntect` works (`InspiredGitHub`, `Solarized (dark)`, `base16-eighties.dark`, …). |
| `save_dir` | string | `"./"` | Default directory pre-filled in the Files sub-tab download prompt (`s`). `~` is expanded; relative paths resolve against the current working directory. |

### Top-level fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `default_workspace` | string | — | Name of the workspace to show when launching the TUI. Must match a `name` in `[[workspaces]]`. If unset, the first workspace is used. |

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
[[workspaces]]
name = "my-workspace"
subscription_id = "00000000-0000-0000-0000-000000000000"
resource_group = "my-rg"
workspace_name = "my-ml-workspace"
region = "eastus2"
```
