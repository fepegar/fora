# Installation

## Install with mise (recommended)

If you use [mise](https://mise.jdx.dev/) for tool management, you can install Fora from the pre-built GitHub releases using the [github backend](https://mise.jdx.dev/dev-tools/backends/github.html):

```bash
mise use -g github:samb-t/fora
```

Or manually add it to your mise config:

```toml
# In your mise.toml
[tools]
"github:samb-t/fora" = "latest"
```

Then run:

```bash
mise install
```

## Quick install

The install script detects your OS and architecture, downloads the right binary from the latest release, and puts it in `~/.local/bin`:

```bash
curl -fsSL https://samb-t.github.io/fora/install.sh | sh
```

Or with `wget`:

```bash
wget -qO- https://samb-t.github.io/fora/install.sh | sh
```

### Options

| Environment variable | Description | Default |
|---|---|---|
| `FORA_VERSION` | Install a specific release tag (e.g. `v0.1.0-20260415-abc1234`) | latest |
| `FORA_INSTALL_DIR` | Directory to install the binary into | `~/.local/bin` |
| `FORA_QUIET` | Suppress informational output (`1` or `true`) | — |
| `FORA_DEBUG` | Show debug output (`1` or `true`) | — |

Pre-built binaries are available for:

- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)

## Prerequisites

- **Azure CLI** — installed and authenticated (`az login`)
- Access to an Azure ML workspace

## Install from source

The following methods require the **Rust toolchain** (1.88 or later). Install it via [rustup](https://rustup.rs/) or a tool manager like [mise](https://mise.jdx.dev/).

### Install with mise (from source)

If you prefer to build from source using mise, you can use the cargo backend:

```bash
mise use -g cargo:https://github.com/samb-t/fora.git
```

Or manually add it to your mise config:

```toml
# In your mise.toml
[tools]
"cargo:https://github.com/samb-t/fora.git" = "latest"
```

Then run:

```bash
mise install
```

### Install with Cargo

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
