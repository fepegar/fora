use std::sync::Arc;

use anyhow::{Context, Result};
use azure_core::credentials::TokenCredential;
use reqwest::Client;

use crate::models::{
    GetMetricHistoryResponse, GetRunResponse, SearchExperimentsRequest,
    SearchExperimentsResponse, SearchRunsRequest, SearchRunsResponse,
};

const MLFLOW_TOKEN_SCOPE: &str = "https://ml.azure.com/.default";

/// MLflow REST API client authenticated via Azure CLI credentials.
#[derive(Clone)]
pub struct MlflowClient {
    http: Client,
    credential: Arc<dyn TokenCredential>,
    base_url: String,
}

impl MlflowClient {
    /// Creates a new MLflow client.
    ///
    /// The base URL is constructed from workspace parameters:
    /// `https://{region}.api.azureml.ms/mlflow/v2.0/subscriptions/{sub}/resourceGroups/{rg}/providers/Microsoft.MachineLearningServices/workspaces/{ws}/api/2.0/mlflow`
    pub fn new(
        credential: Arc<dyn TokenCredential>,
        region: &str,
        subscription_id: &str,
        resource_group: &str,
        workspace_name: &str,
    ) -> Self {
        let base_url = format!(
            "https://{region}.api.azureml.ms/mlflow/v2.0/subscriptions/{sub}/resourceGroups/{rg}/providers/Microsoft.MachineLearningServices/workspaces/{ws}/api/2.0/mlflow",
            region = region,
            sub = subscription_id,
            rg = resource_group,
            ws = workspace_name,
        );

        Self {
            http: Client::new(),
            credential,
            base_url,
        }
    }

    async fn get_token(&self) -> Result<String> {
        let token = self
            .credential
            .get_token(&[MLFLOW_TOKEN_SCOPE], None)
            .await
            .context("Failed to acquire Azure token for MLflow")?;
        Ok(token.token.secret().to_string())
    }

    /// Search experiments. Returns a page of experiments plus an optional continuation token.
    pub async fn search_experiments(
        &self,
        max_results: Option<u32>,
        page_token: Option<&str>,
        order_by: Option<Vec<String>>,
        filter: Option<String>,
    ) -> Result<SearchExperimentsResponse> {
        let url = format!("{}/experiments/search", self.base_url);

        let body = SearchExperimentsRequest {
            max_results,
            page_token: page_token.map(|s| s.to_string()),
            filter,
            order_by,
        };

        let token = self.get_token().await?;
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .context("MLflow experiments/search request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("MLflow experiments/search returned {}: {}", status, text);
        }

        resp.json::<SearchExperimentsResponse>()
            .await
            .context("Failed to parse experiments/search response")
    }

    /// Fetch all experiments across all pages.
    pub async fn search_all_experiments(&self) -> Result<Vec<crate::models::Experiment>> {
        let mut all = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let resp = self
                .search_experiments(Some(1000), page_token.as_deref(), None, None)
                .await?;
            all.extend(resp.experiments);

            match resp.next_page_token {
                Some(token) if !token.is_empty() => {
                    page_token = Some(token);
                }
                _ => break,
            }
        }

        Ok(all)
    }

    /// Search runs across experiments. Returns a page of runs plus an optional continuation token.
    pub async fn search_runs(&self, request: &SearchRunsRequest) -> Result<SearchRunsResponse> {
        let url = format!("{}/runs/search", self.base_url);

        let token = self.get_token().await?;
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(request)
            .send()
            .await
            .context("MLflow runs/search request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("MLflow runs/search returned {}: {}", status, text);
        }

        resp.json::<SearchRunsResponse>()
            .await
            .context("Failed to parse runs/search response")
    }

    /// Get a single run by ID, returning its full data including current metrics.
    pub async fn get_run(&self, run_id: &str) -> Result<crate::models::Run> {
        let url = format!("{}/runs/get", self.base_url);

        let token = self.get_token().await?;
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .query(&[("run_id", run_id)])
            .send()
            .await
            .context("MLflow runs/get request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("MLflow runs/get returned {}: {}", status, text);
        }

        let response: GetRunResponse = resp
            .json()
            .await
            .context("Failed to parse runs/get response")?;

        Ok(response.run)
    }

    /// Get metric history for a single metric key on a run.
    pub async fn get_metric_history(
        &self,
        run_id: &str,
        metric_key: &str,
        max_results: Option<u32>,
        page_token: Option<&str>,
    ) -> Result<GetMetricHistoryResponse> {
        let url = format!("{}/metrics/get-history", self.base_url);

        let token = self.get_token().await?;
        let mut req = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .query(&[("run_id", run_id), ("metric_key", metric_key)]);

        if let Some(max) = max_results {
            req = req.query(&[("max_results", &max.to_string())]);
        }
        if let Some(pt) = page_token {
            req = req.query(&[("page_token", pt)]);
        }

        let resp = req
            .send()
            .await
            .context("MLflow metrics/get-history request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("MLflow metrics/get-history returned {}: {}", status, text);
        }

        resp.json::<GetMetricHistoryResponse>()
            .await
            .context("Failed to parse metrics/get-history response")
    }

    /// Fetch all pages of metric history for a single metric key on a run.
    pub async fn get_all_metric_history(
        &self,
        run_id: &str,
        metric_key: &str,
    ) -> Result<Vec<crate::models::Metric>> {
        let mut all = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let resp = self
                .get_metric_history(run_id, metric_key, None, page_token.as_deref())
                .await?;
            all.extend(resp.metrics);

            match resp.next_page_token {
                Some(token) if !token.is_empty() => {
                    page_token = Some(token);
                }
                _ => break,
            }
        }

        Ok(all)
    }
}
