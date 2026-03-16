use anyhow::Result;
use futures::StreamExt;

use azure_ml::models::{Compute, ComputeResource};

use crate::client::AzureClient;

use super::state::ComputeRow;

/// Fetches all compute resources from the workspace.
pub async fn fetch_compute(client: &AzureClient) -> Result<Vec<ComputeRow>> {
    let compute_client = client.compute();
    let mut pager = compute_client.list(
        client.resource_group(),
        client.workspace_name(),
        None,
    )?;

    let mut rows = Vec::new();
    while let Some(result) = pager.next().await {
        let resource = result?;
        rows.push(map_compute_resource(resource));
    }

    Ok(rows)
}

fn map_compute_resource(resource: ComputeResource) -> ComputeRow {
    let name = resource.name.unwrap_or_default();
    let location = resource.location;

    match resource.properties {
        Some(Compute::AmlCompute(aml)) => {
            let (
                vm_size,
                current_node_count,
                min_nodes,
                max_nodes,
                idle_nodes,
                running_nodes,
                preparing_nodes,
                unusable_nodes,
                allocation_state,
                vm_priority,
            ) = aml
                .properties
                .map(|p| {
                    let node_counts = p.node_state_counts.as_ref();
                    let scale = p.scale_settings.as_ref();
                    (
                        p.vm_size,
                        p.current_node_count,
                        scale.and_then(|s| s.min_node_count),
                        scale.and_then(|s| s.max_node_count),
                        node_counts.and_then(|n| n.idle_node_count),
                        node_counts.and_then(|n| n.running_node_count),
                        node_counts.and_then(|n| n.preparing_node_count),
                        node_counts.and_then(|n| n.unusable_node_count),
                        p.allocation_state,
                        p.vm_priority.map(|v| format!("{:?}", v)),
                    )
                })
                .unwrap_or_default();

            ComputeRow {
                name,
                compute_type: "AmlCompute".to_string(),
                vm_size,
                provisioning_state: aml.provisioning_state,
                allocation_state,
                current_node_count,
                min_nodes,
                max_nodes,
                idle_nodes,
                running_nodes,
                preparing_nodes,
                unusable_nodes,
                location,
                vm_priority,
                description: aml.description,
            }
        }
        Some(Compute::ComputeInstance(ci)) => ComputeRow {
            name,
            compute_type: "ComputeInstance".to_string(),
            vm_size: ci.properties.and_then(|p| p.vm_size),
            provisioning_state: ci.provisioning_state,
            allocation_state: None,
            current_node_count: None,
            min_nodes: None,
            max_nodes: None,
            idle_nodes: None,
            running_nodes: None,
            preparing_nodes: None,
            unusable_nodes: None,
            location,
            vm_priority: None,
            description: ci.description,
        },
        _ => ComputeRow {
            name,
            compute_type: "Other".to_string(),
            vm_size: None,
            provisioning_state: None,
            allocation_state: None,
            current_node_count: None,
            min_nodes: None,
            max_nodes: None,
            idle_nodes: None,
            running_nodes: None,
            preparing_nodes: None,
            unusable_nodes: None,
            location,
            vm_priority: None,
            description: None,
        },
    }
}
