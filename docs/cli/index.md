# CLI Overview

The `fora` command is the main entry point. Running it with no subcommand starts the interactive terminal UI. Subcommands are available for specific tasks.

## Usage

```bash
# Start the TUI
fora

# Run a subcommand
fora <command> [options]
```

## Commands

| Command | Description |
|---------|-------------|
| *(none)* | Start the interactive terminal UI |
| [`submit`](./submit) | Submit a job to Azure ML |

## Terminal UI

When launched without a subcommand, Fora starts an interactive TUI with the following tabs:

- **Recent Jobs** — View and filter your recent ML jobs
- **Experiments** — Browse experiments and their runs
- **Compute** — Monitor compute cluster status

The TUI supports keyboard navigation, search and filtering, a column picker for customising table layouts, and a workspace switcher for managing multiple Azure ML workspaces.

### Key bindings

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch between tabs |
| `↑` / `↓` | Navigate rows |
| `/` | Open search |
| `c` | Open column picker |
| `w` | Open workspace picker |
| `Enter` | View job details |
| `?` | Toggle help bar |
| `q` | Quit |

## Environment variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Control log verbosity (e.g., `RUST_LOG=debug fora`) |
