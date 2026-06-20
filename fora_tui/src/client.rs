use std::sync::Arc;

use anyhow::Result;
use azure_identity::AzureCliCredential;
use azure_ml::MachineLearningServicesClient;
use mlflow::{ArtifactsClient, MlflowClient};

use crate::config::WorkspaceConfig;

/// Wraps the Azure ML client with workspace context.
#[derive(Clone)]
pub struct AzureClient {
    inner: Arc<MachineLearningServicesClient>,
    mlflow_client: MlflowClient,
    artifacts_client: ArtifactsClient,
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
            credential.clone(),
            &workspace.region,
            &workspace.subscription_id,
            &workspace.resource_group,
            &workspace.workspace_name,
        );

        let artifacts_client = ArtifactsClient::new(
            credential,
            &workspace.region,
            &workspace.subscription_id,
            &workspace.resource_group,
            &workspace.workspace_name,
        );

        Ok(Self {
            inner: Arc::new(client),
            mlflow_client,
            artifacts_client,
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

    pub fn artifacts(&self) -> &ArtifactsClient {
        &self.artifacts_client
    }

    pub fn resource_group(&self) -> &str {
        &self.workspace.resource_group
    }

    pub fn workspace_name(&self) -> &str {
        &self.workspace.workspace_name
    }

    /// Cancel a running job by its ID.
    ///
    /// The cancel API returns HTTP 202 with an empty body. The auto-generated
    /// Poller tries to deserialize that empty body as JSON, which fails with a
    /// serde "EOF while parsing" error. Since the HTTP request already succeeded
    /// at that point (the pipeline validates the status code), we treat this
    /// specific deserialization error as a successful cancellation.
    pub async fn cancel_job(&self, job_id: &str) -> Result<()> {
        let poller =
            self.jobs()
                .cancel(self.resource_group(), self.workspace_name(), job_id, None)?;
        match poller.await {
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("EOF while parsing a value") => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
