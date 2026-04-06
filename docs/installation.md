# Installation

Fora does not have pre-built binaries yet, so it must be installed from source.

## Prerequisites

- **Rust toolchain** — 1.70 or later ([install via rustup](https://rustup.rs/))
- **Azure CLI** — installed and authenticated (`az login`)
- Access to an Azure ML workspace

## Install from source with Cargo

Clone the repository and install the `fora` binary:

```bash
git clone https://github.com/samb-t/fora.git
cd fora
cargo install --path fora_cli
```

This builds the `fora` binary and places it in your Cargo bin directory (usually `~/.cargo/bin/`). Make sure this directory is on your `PATH`.

To verify the installation:

```bash
fora --help
```

## Install with mise

If you use [mise](https://mise.jdx.dev/) for tool management, you can install Fora using the cargo backend:

```toml
# In your mise.toml or .mise.toml
[tools]
"cargo:fora_cli" = { git = "https://github.com/samb-t/fora.git" }
```

Then run:

```bash
mise install
```

## Updating

To update to the latest version, pull the latest source and reinstall:

```bash
cd fora
git pull
cargo install --path fora_cli
```
