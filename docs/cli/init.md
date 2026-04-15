# fora init

Interactively set up a Fora configuration file by discovering your Azure resources.

## Usage

```bash
fora init
```

If no configuration file is found when starting the TUI (`fora` with no arguments), the init wizard runs automatically.

## What it does

The wizard walks you through the following steps:

1. **Fetch your Azure profile** — Retrieves your display name from Microsoft Graph.
2. **Select subscriptions** — Lists all Azure subscriptions accessible to your account and lets you pick which ones to use. Press <kbd>Space</kbd> to select, <kbd>Enter</kbd> to confirm.
3. **Select ML workspaces** — Lists all Azure ML workspaces across all resource groups in the selected subscriptions. Press <kbd>Space</kbd> to select, <kbd>Enter</kbd> to confirm.
4. **Select default workspace** — If more than one workspace is selected, lets you choose which workspace to show when launching the TUI.
5. **Choose config location** — Save to `~/.config/fora.toml` (global) or `./fora.toml` (local).

## Prerequisites

You must be logged in to Azure before running the wizard:

```bash
az login
```
