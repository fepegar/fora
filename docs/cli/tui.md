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

### Detail pane

Pressing `Enter` on a job opens the detail pane. It exposes three
sub-tabs, switchable with their number keys:

| Key | Action |
|-----|--------|
| `1` | Switch to the Info sub-tab |
| `2` | Switch to the Metrics sub-tab |
| `3` | Switch to the Files sub-tab |
| `Esc` | Close the detail pane |

### Files sub-tab

The Files sub-tab browses the run's artifact tree and previews the
selected file in-pane. Text files are syntax-highlighted; logs keep their
ANSI colours (e.g. from `tqdm`, `rich`) or get lightweight log-level and
traceback highlighting; and image artifacts are rendered using the
terminal's best available graphics protocol
(Kitty, iTerm2, Sixel, or a Unicode half-block fallback).

While a run is active, the previewed file is live-tailed every
`file_preview_refresh_secs` seconds.

| Key | Action |
|-----|--------|
| `←` / `→` | Move focus between the tree and the preview pane |
| `↑` / `↓`, `j` / `k` | Navigate within the focused pane |
| `Enter` / `Space` | Open the highlighted directory or load the highlighted file into the preview |
| `Backspace` | Collapse the current directory or go up one level |
| `PgUp` / `PgDn` | Page the focused pane |
| `Home` / `g` | Jump to the top of the focused pane |
| `End` / `G` | Jump to the bottom of the focused pane |
| `s` | Download the highlighted file or directory to disk |
| `r` | Force-refresh the current directory and previewed file |
| `z` | Toggle fullscreen preview (hides everything except the file content; `Esc` exits fullscreen) |

Pressing `s` prompts for a destination path, pre-filled with
`<save_dir>/<name>` (see [`save_dir`](../setup)). Edit it to
rename or relocate, then press `Enter`. Directories download recursively,
preserving their structure. If the destination already exists you are
asked to confirm before it is overwritten.

## Environment variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Control log verbosity (e.g., `RUST_LOG=debug fora`) |
