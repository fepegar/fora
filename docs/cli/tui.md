# fora

Start the interactive terminal UI for browsing Azure ML workspaces.

## Usage

```bash
fora
```

When launched without a subcommand, Fora starts an interactive TUI with the following tabs:

- **Recent Jobs** — View and filter your recent ML jobs
- **Experiments** — Browse experiments and their runs
- **Compute** — Monitor compute cluster status

The TUI supports keyboard navigation, search and filtering, a column picker for customising table layouts, and a workspace switcher for managing multiple Azure ML workspaces.

## Key bindings

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
