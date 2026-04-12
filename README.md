# Fora

> [!WARNING]
> Fora is actively in development. Expect breaking changes, incomplete features, and rough edges.

Azure ML from your terminal.

Fora is a terminal UI for managing [Azure Machine Learning](https://azure.microsoft.com/en-us/products/machine-learning) workspaces. Browse jobs, experiments, and compute in an interactive TUI.

## Features

- **Terminal UI** — Browse recent jobs, experiments, and compute resources in an interactive interface built with [Ratatui](https://ratatui.rs/)
- **Multi-workspace** — Switch between Azure ML workspaces on the fly
- **TOML configuration** — Simple config for workspaces, UI preferences, and column layouts

## Install

Requires the [Rust toolchain](https://rustup.rs/) and [Azure CLI](https://learn.microsoft.com/en-us/cli/azure/install-azure-cli) (`az login`).

```bash
git clone https://github.com/samb-t/fora.git
cd fora
cargo install --path fora_cli
```

See the [installation docs](https://samb-t.github.io/fora/installation) for other methods including [mise](https://mise.jdx.dev/).

## Quick start

Launch the TUI and set up a config interactively:

```bash
fora
```

Or manually run the configuration setup:

```bash
fora init
```

## Documentation

Full documentation is available at **[samb-t.github.io/fora](https://samb-t.github.io/fora/)**.

## Development

```bash
cargo build                                    # build
cargo test --all                               # test
cargo clippy --all-targets --all-features      # lint
cargo fmt --all                                # format
```

## License

MIT
