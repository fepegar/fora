# Fora - Azure ML Management Tools

A collection of Rust tools for managing Azure Machine Learning workspaces, built as a Cargo workspace.

## Workspace Structure

This project is organized as a Rust workspace with the following crates:

### `azure_ml`
A library crate providing:
- High-level Azure ML REST API client (`AzureMLClient`)
- Authentication handling using Azure Identity
- Type-safe operations for common Azure ML resources

### `fora_tui` 
A terminal user interface application for managing Azure ML workspaces, featuring:
- Interactive browsing of jobs, models, environments, and data assets
- Real-time status monitoring
- Tabbed interface with keyboard navigation
- Built with [Ratatui](https://ratatui.rs/)

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Azure CLI installed and authenticated, or other Azure credential provider
- Access to an Azure ML workspace

### Building

```bash
# Build the entire workspace
cargo build

# Build specific crates
cargo build -p azure_ml
cargo build -p fora_tui
```

### Running the TUI

```bash
# Run the terminal interface
cargo run -p fora_tui

# Or install and run
cargo install --path fora_tui
fora_tui
```

### Configuration

The TUI application will prompt for Azure configuration on first run:
- Subscription ID
- Resource Group
- Workspace Name

Configuration is stored in your system's config directory.

## Development

### Testing

```bash
# Test the entire workspace
cargo test

# Test specific crates
cargo test -p azure_ml
cargo test -p fora_tui
```

### Adding Dependencies

Dependencies are managed at the workspace level in the root `Cargo.toml`. Add shared dependencies to the `[workspace.dependencies]` section, then reference them in individual crate `Cargo.toml` files using `{ workspace = true }`.

## Architecture

### Azure ML Client

The `azure_ml` crate provides a high-level client that abstracts Azure ML REST API operations:

```rust
use azure_ml::{AzureMLClient, AzureMLConfig};

let config = AzureMLConfig {
    subscription_id: "your-subscription-id".to_string(),
    resource_group_name: "your-rg".to_string(),
    workspace_name: "your-workspace".to_string(),
    ..Default::default()
};

let client = AzureMLClient::new(config)?;
let jobs = client.list_jobs().await?;
```

### TUI Application

The `fora_tui` crate implements a tabbed interface with:
- **Home**: Workspace overview and quick actions
- **Jobs**: Browse and monitor ML jobs
- **Models**: Manage model versions
- **Compute**: View compute resources
- **Data**: Browse data assets

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests where appropriate
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Future Plans

- CLI interface (`fora_cli` crate)
- Additional Azure ML resource types
- Integration with MLflow
- Deployment management
- Cost tracking and optimization