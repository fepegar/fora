use std::sync::Arc;

use anyhow::Result;
use azure_identity::AzureCliCredential;
use azure_ml::MachineLearningServicesClient;
use mlflow::MlflowClient;

use crate::config::WorkspaceConfig;

/// Wraps the Azure ML client with workspace context.
#[derive(Clone)]
pub struct AzureClient {
    inner: Arc<MachineLearningServicesClient>,
    mlflow_client: MlflowClient,
    pub workspace: WorkspaceConfig,
}

impl AzureClient {
    pub fn new(workspace: WorkspaceConfig) -> Result<Self> {
        let credential = AzureCliCredential::new(None)?;

        let client = MachineLearningServicesClient::new(
            "https://management.azure.com",
            credential.clone(),
            workspace.subscription_id.clone(),
            None,
        )?;

        let mlflow_client = MlflowClient::new(
            credential,
            &workspace.region,
            &workspace.subscription_id,
            &workspace.resource_group,
            &workspace.workspace_name,
        );

        Ok(Self {
            inner: Arc::new(client),
            mlflow_client,
            workspace,
        })
    }

    pub fn jobs(&self) -> azure_ml::clients::MachineLearningServicesJobsClient {
        self.inner.get_machine_learning_services_jobs_client()
    }

    pub fn compute(&self) -> azure_ml::clients::MachineLearningServicesComputeClient {
        self.inner.get_machine_learning_services_compute_client()
    }

    pub fn mlflow(&self) -> &MlflowClient {
        &self.mlflow_client
    }

    pub fn resource_group(&self) -> &str {
        &self.workspace.resource_group
    }

    pub fn workspace_name(&self) -> &str {
        &self.workspace.workspace_name
    }
}
