use azure_ml::models::{AllocationState, ProvisioningState};

/// Flattened representation of a compute resource for display.
#[derive(Debug, Clone)]
pub struct ComputeRow {
    pub name: String,
    pub compute_type: String,
    pub vm_size: Option<String>,
    pub provisioning_state: Option<ProvisioningState>,
    pub allocation_state: Option<AllocationState>,
    pub current_node_count: Option<i32>,
    pub min_nodes: Option<i32>,
    pub max_nodes: Option<i32>,
    pub idle_nodes: Option<i32>,
    pub running_nodes: Option<i32>,
    pub preparing_nodes: Option<i32>,
    pub unusable_nodes: Option<i32>,
    pub location: Option<String>,
    pub vm_priority: Option<String>,
    pub description: Option<String>,
}

/// State for the Compute tab.
#[derive(Debug, Default)]
pub struct ComputeState {
    pub detail_open: bool,
}
