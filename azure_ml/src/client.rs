use azure_core::error::Result as AzureResult;
use azure_core::http::{Method, Pipeline, Request, Url};
use azure_core::Error as AzureError;
use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use std::sync::Arc;

use crate::models;

#[derive(Clone)]
pub struct AzureMLClient {
    endpoint: String,
    subscription_id: String,
    pipeline: Arc<Pipeline>,
    api_version: String,
}

impl AzureMLClient {
    /// Create a new Azure ML client
    pub fn new(
        endpoint: String,
        subscription_id: String,
        pipeline: Arc<Pipeline>,
        api_version: Option<String>,
    ) -> Self {
        Self {
            endpoint,
            subscription_id,
            pipeline,
            api_version: api_version.unwrap_or_else(|| "2025-09-01".to_string()),
        }
    }

    /// Create a new Azure ML client from pipeline (use azure_core to create the pipeline)
    pub fn from_pipeline(
        endpoint: String,
        subscription_id: String,
        pipeline: Pipeline,
        api_version: Option<String>,
    ) -> Self {
        Self {
            endpoint,
            subscription_id,
            pipeline: Arc::new(pipeline),
            api_version: api_version.unwrap_or_else(|| "2025-09-01".to_string()),
        }
    }

    /// Generic send helper for making HTTP requests
    pub async fn send<TReq, TResp>(
        &self,
        method: Method,
        url: Url,
        body: Option<TReq>,
    ) -> AzureResult<TResp>
    where
        TReq: Serialize + Send + Sync,
        TResp: DeserializeOwned + Send + Sync + 'static,
    {
        let mut req = Request::new(url, method);

        // Set body if present
        if let Some(b) = body {
            let bytes = serde_json::to_vec(&b).map_err(|e| {
                AzureError::message(
                    azure_core::error::ErrorKind::DataConversion,
                    format!("Failed to serialize request body: {}", e),
                )
            })?;
            req.set_body(bytes.into());
            req.headers_mut()
                .insert("content-type", "application/json".parse().unwrap());
        }

        // Send request through pipeline
        let response = self.pipeline.send(&mut req, &Default::default()).await?;

        // Read response body
        let body_bytes = response.into_body().collect().await.map_err(|e| {
            AzureError::message(
                azure_core::error::ErrorKind::HttpResponse,
                format!("Failed to read response body: {}", e),
            )
        })?;

        // Deserialize response
        let result: TResp = serde_json::from_slice(&body_bytes).map_err(|e| {
            AzureError::message(
                azure_core::error::ErrorKind::DataConversion,
                format!("Failed to deserialize response: {}", e),
            )
        })?;

        Ok(result)
    }

    /// Create or update a job
    pub async fn create_or_update_job(
        &self,
        resource_group: &str,
        workspace: &str,
        job_id: &str,
        body: models::Job,
    ) -> AzureResult<models::JobBaseResource> {
        let url_str = format!(
            "{}/subscriptions/{}/resourceGroups/{}/providers/Microsoft.MachineLearningServices/workspaces/{}/jobs/{}?api-version={}",
            self.endpoint, self.subscription_id, resource_group, workspace, job_id, self.api_version
        );

        let url = Url::parse(&url_str).map_err(|e| {
            AzureError::message(
                azure_core::error::ErrorKind::Other,
                format!("Invalid URL: {}", e),
            )
        })?;

        self.send(Method::Put, url, Some(body)).await
    }

    /// List jobs in the workspace
    pub async fn list_jobs(
        &self,
        resource_group: &str,
        workspace: &str,
        skip: Option<i32>,
        job_type: Option<&str>,
        tag: Option<&str>,
        list_view_type: Option<models::ListViewType>,
        properties: Option<&str>,
    ) -> AzureResult<models::JobBaseResourceArmPaginatedResult> {
        let mut url_str = format!(
            "{}/subscriptions/{}/resourceGroups/{}/providers/Microsoft.MachineLearningServices/workspaces/{}/jobs?api-version={}",
            self.endpoint, self.subscription_id, resource_group, workspace, self.api_version
        );

        // Build query parameters
        let mut query_params = Vec::new();

        if let Some(skip_val) = skip {
            query_params.push(format!("$skip={}", skip_val));
        }

        if let Some(job_type_val) = job_type {
            query_params.push(format!("jobType={}", job_type_val));
        }

        if let Some(tag_val) = tag {
            query_params.push(format!("tag={}", tag_val));
        }

        if let Some(list_view_type_val) = list_view_type {
            query_params.push(format!("listViewType={}", list_view_type_val));
        }

        if let Some(properties_val) = properties {
            query_params.push(format!("properties={}", properties_val));
        }

        // Append query parameters if any exist
        if !query_params.is_empty() {
            url_str.push('&');
            url_str.push_str(&query_params.join("&"));
        }

        let url = Url::parse(&url_str).map_err(|e| {
            AzureError::message(
                azure_core::error::ErrorKind::Other,
                format!("Invalid URL: {}", e),
            )
        })?;

        self.send(Method::Get, url, None::<()>).await
    }
}
