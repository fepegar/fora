use std::sync::Arc;

use anyhow::Result;
use azure_identity::AzureCliCredential;
use azure_ml::MachineLearningServicesClient;

use crate::config::WorkspaceConfig;

/// Wraps the Azure ML client with workspace context.
#[derive(Clone)]
pub struct AzureClient {
    inner: Arc<MachineLearningServicesClient>,
    pub workspace: WorkspaceConfig,
}

impl AzureClient {
    pub fn new(workspace: WorkspaceConfig) -> Result<Self> {
        let credential = AzureCliCredential::new(None)?;
        let client = MachineLearningServicesClient::new(
            "https://management.azure.com",
            credential,
            workspace.subscription_id.clone(),
            None,
        )?;

        Ok(Self {
            inner: Arc::new(client),
            workspace,
        })
    }

    pub fn jobs(
        &self,
    ) -> azure_ml::clients::MachineLearningServicesJobsClient {
        self.inner.get_machine_learning_services_jobs_client()
    }

    pub fn compute(
        &self,
    ) -> azure_ml::clients::MachineLearningServicesComputeClient {
        self.inner.get_machine_learning_services_compute_client()
    }

    pub fn resource_group(&self) -> &str {
        &self.workspace.resource_group
    }

    pub fn workspace_name(&self) -> &str {
        &self.workspace.workspace_name
    }
}
