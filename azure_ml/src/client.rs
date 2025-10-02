use azure_core::error::Result as AzureResult;
use azure_core::http::{Context, Method, Pipeline, Request, Response, Url};
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
    /// Generic send helper:
    /// - TReq: serializable request body (Option)
    /// - TResp: deserializable response body
    pub async fn send<TReq, TResp>(
        &self,
        method: Method,
        url: Url,
        body: Option<&TReq>,
        ctx: Option<&Context>,
    ) -> AzureResult<TResp>
    where
        TReq: Serialize + Sync,
        TResp: DeserializeOwned + Send + Sync + 'static,
    {
        // Build request
        let mut req = Request::new(url, method);

        // Set body if present and add header
        if let Some(b) = body {
            let bytes = serde_json::to_vec(b).map_err(|e| {
                AzureError::message(
                    azure_core::error::ErrorKind::DataConversion,
                    format!("serialize body: {}", e),
                )
            })?;
            req.set_body(bytes.into());
            req.headers_mut()
                .insert("content-type", "application/json".parse().unwrap());
        }

        // Prepare context
        let context = ctx.cloned().unwrap_or_default();

        // Send through pipeline.
        // NOTE: the exact pipeline API can differ across azure_core versions.
        // Here we use a conceptual `send` that accepts `&mut Request` and `&Context`.
        let resp: Response = self
            .pipeline
            .send(&mut req, &context) // adapt call if your azure_core API differs
            .await
            .map_err(|e| {
                AzureError::message(azure_core::error::ErrorKind::Other, format!("{:?}", e))
            })?;

        // Read response body bytes
        let body_bytes = resp.into_body().collect().await.map_err(|e| {
            AzureError::message(
                azure_core::error::ErrorKind::HttpResponse,
                format!("{:?}", e),
            )
        })?;

        // Deserialize into the requested type
        let result: TResp = serde_json::from_slice(&body_bytes).map_err(|e| {
            AzureError::message(
                azure_core::error::ErrorKind::DataConversion,
                format!("deserialize: {}", e),
            )
        })?;

        Ok(result)
    }

    /// Example: very thin create_or_update_job that uses the generic send helper.
    pub async fn create_or_update_job(
        &self,
        resource_group: &str,
        workspace: &str,
        job_id: &str,
        body: &models::CreateJobRequest, // generated model
    ) -> AzureResult<models::JobBaseResource> {
        let url_str = format!(
            "{}/subscriptions/{}/resourceGroups/{}/providers/Microsoft.MachineLearningServices/workspaces/{}/jobs/{}?api-version={}",
            self.endpoint, self.subscription_id, resource_group, workspace, job_id, self.api_version
        );
        let url = Url::parse(&url_str).map_err(|e| {
            AzureError::message(
                azure_core::error::ErrorKind::Other,
                format!("url parse: {}", e),
            )
        })?;

        self.send(Method::Put, url, Some(body), None).await
    }
}
