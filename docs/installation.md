# Installation

Fora does not have pre-built binaries yet, so it must be installed from source.

## Prerequisites

- **Rust toolchain** — 1.70 or later (install via [rustup](https://rustup.rs/) or a tool manager like [mise](https://mise.jdx.dev/))
- **Azure CLI** — installed and authenticated (`az login`)
- Access to an Azure ML workspace

## Install with mise

If you use [mise](https://mise.jdx.dev/) for tool management, you can install Fora using the cargo backend:

To install `fora` globally, run the following command

```bash
mise use -g cargo:https://github.com/samb-t/fora.git
```

Or manually add the following to your mise config:

```toml
# In your mise.toml
[tools]
"cargo:fora_cli" = { git = "https://github.com/samb-t/fora.git" }
```

Then run:

```bash
mise install
```

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
