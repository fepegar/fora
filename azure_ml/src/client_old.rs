//! Azure ML REST API client
//!
//! This module provides a high-level client for interacting with Azure ML services.

use crate::{AzureMLError, Result};
use azure_core::auth::TokenCredential;
use azure_identity::DefaultAzureCredential;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

/// Configuration for the Azure ML client
#[derive(Debug, Clone)]
pub struct AzureMLConfig {
    /// Azure subscription ID
    pub subscription_id: String,
    /// Resource group name
    pub resource_group_name: String,
    /// Azure ML workspace name
    pub workspace_name: String,
    /// Azure ML API version (defaults to "2023-10-01")
    pub api_version: String,
    /// Base URL for Azure ML API (defaults to Azure public cloud)
    pub base_url: String,
}

impl Default for AzureMLConfig {
    fn default() -> Self {
        Self {
            subscription_id: String::new(),
            resource_group_name: String::new(),
            workspace_name: String::new(),
            api_version: "2023-10-01".to_string(),
            base_url: "https://management.azure.com".to_string(),
        }
    }
}

/// High-level client for Azure ML REST API
pub struct AzureMLClient {
    config: AzureMLConfig,
    credential: Arc<dyn TokenCredential>,
    http_client: Client,
}

impl AzureMLClient {
    /// Create a new Azure ML client with default Azure credentials
    pub fn new(config: AzureMLConfig) -> Result<Self> {
        let credential = Arc::new(DefaultAzureCredential::create(Default::default())?);
        let http_client = Client::new();

        // Validate required configuration
        if config.subscription_id.is_empty() {
            return Err(AzureMLError::Configuration(
                "subscription_id is required".to_string(),
            ));
        }
        if config.resource_group_name.is_empty() {
            return Err(AzureMLError::Configuration(
                "resource_group_name is required".to_string(),
            ));
        }
        if config.workspace_name.is_empty() {
            return Err(AzureMLError::Configuration(
                "workspace_name is required".to_string(),
            ));
        }

        Ok(Self {
            config,
            credential,
            http_client,
        })
    }

    /// Create a new Azure ML client with custom credentials
    pub fn with_credential(
        config: AzureMLConfig,
        credential: Arc<dyn TokenCredential>,
    ) -> Result<Self> {
        let http_client = Client::new();

        // Validate required configuration
        if config.subscription_id.is_empty() {
            return Err(AzureMLError::Configuration(
                "subscription_id is required".to_string(),
            ));
        }
        if config.resource_group_name.is_empty() {
            return Err(AzureMLError::Configuration(
                "resource_group_name is required".to_string(),
            ));
        }
        if config.workspace_name.is_empty() {
            return Err(AzureMLError::Configuration(
                "workspace_name is required".to_string(),
            ));
        }

        Ok(Self {
            config,
            credential,
            http_client,
        })
    }

    /// Get the base URL for workspace-scoped operations
    fn workspace_base_url(&self) -> String {
        format!(
            "{}/subscriptions/{}/resourceGroups/{}/providers/Microsoft.MachineLearningServices/workspaces/{}",
            self.config.base_url,
            self.config.subscription_id,
            self.config.resource_group_name,
            self.config.workspace_name
        )
    }

    /// Build a URL for a specific resource path
    fn build_url(&self, path: &str) -> Result<Url> {
        let base = self.workspace_base_url();
        let full_path = if path.starts_with('/') {
            format!("{}{}", base, path)
        } else {
            format!("{}/{}", base, path)
        };

        let mut url = Url::parse(&full_path)
            .map_err(|e| AzureMLError::Configuration(format!("Invalid URL: {}", e)))?;

        url.query_pairs_mut()
            .append_pair("api-version", &self.config.api_version);

        Ok(url)
    }

    /// Get an access token for Azure ML API
    async fn get_access_token(&self) -> Result<String> {
        let token_response = self
            .credential
            .get_token(&["https://management.azure.com/.default"])
            .await?;

        Ok(token_response.token.secret().to_string())
    }

    /// Create an authenticated HTTP request builder
    async fn request(&self, method: reqwest::Method, url: Url) -> Result<RequestBuilder> {
        let token = self.get_access_token().await?;

        Ok(self
            .http_client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json"))
    }

    /// Send a GET request and deserialize the response
    pub async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = self.build_url(path)?;
        let response = self
            .request(reqwest::Method::GET, url)
            .await?
            .send()
            .await?;

        if response.status().is_success() {
            let json_response = response.json::<T>().await?;
            Ok(json_response)
        } else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(AzureMLError::Api {
                status_code,
                message: error_text,
            })
        }
    }

    /// Send a POST request with a JSON body and deserialize the response
    pub async fn post<T, U>(&self, path: &str, body: &T) -> Result<U>
    where
        T: Serialize,
        U: for<'de> Deserialize<'de>,
    {
        let url = self.build_url(path)?;
        let response = self
            .request(reqwest::Method::POST, url)
            .await?
            .json(body)
            .send()
            .await?;

        if response.status().is_success() {
            let json_response = response.json::<U>().await?;
            Ok(json_response)
        } else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(AzureMLError::Api {
                status_code,
                message: error_text,
            })
        }
    }

    /// Send a PUT request with a JSON body and deserialize the response
    pub async fn put<T, U>(&self, path: &str, body: &T) -> Result<U>
    where
        T: Serialize,
        U: for<'de> Deserialize<'de>,
    {
        let url = self.build_url(path)?;
        let response = self
            .request(reqwest::Method::PUT, url)
            .await?
            .json(body)
            .send()
            .await?;

        if response.status().is_success() {
            let json_response = response.json::<U>().await?;
            Ok(json_response)
        } else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(AzureMLError::Api {
                status_code,
                message: error_text,
            })
        }
    }

    /// Send a DELETE request
    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = self.build_url(path)?;
        let response = self
            .request(reqwest::Method::DELETE, url)
            .await?
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status_code = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            Err(AzureMLError::Api {
                status_code,
                message: error_text,
            })
        }
    }

    /// Get workspace information
    pub async fn get_workspace(&self) -> Result<serde_json::Value> {
        self.get("").await
    }

    /// List jobs in the workspace
    pub async fn list_jobs(&self) -> Result<serde_json::Value> {
        self.get("/jobs").await
    }

    /// Get a specific job by name
    pub async fn get_job(&self, job_name: &str) -> Result<serde_json::Value> {
        let path = format!("/jobs/{}", job_name);
        self.get(&path).await
    }

    /// List models in the workspace
    pub async fn list_models(&self) -> Result<serde_json::Value> {
        self.get("/models").await
    }

    /// Get a specific model by name and version
    pub async fn get_model(&self, model_name: &str, version: &str) -> Result<serde_json::Value> {
        let path = format!("/models/{}/versions/{}", model_name, version);
        self.get(&path).await
    }

    /// List environments in the workspace
    pub async fn list_environments(&self) -> Result<serde_json::Value> {
        self.get("/environments").await
    }

    /// Get a specific environment by name and version
    pub async fn get_environment(
        &self,
        environment_name: &str,
        version: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/environments/{}/versions/{}", environment_name, version);
        self.get(&path).await
    }

    /// List data assets in the workspace
    pub async fn list_data_assets(&self) -> Result<serde_json::Value> {
        self.get("/data").await
    }

    /// Get a specific data asset by name and version
    pub async fn get_data_asset(
        &self,
        data_name: &str,
        version: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/data/{}/versions/{}", data_name, version);
        self.get(&path).await
    }

    /// List batch endpoints in the workspace
    pub async fn list_batch_endpoints(&self) -> Result<serde_json::Value> {
        self.get("/batchEndpoints").await
    }

    /// Get a specific batch endpoint by name
    pub async fn get_batch_endpoint(&self, endpoint_name: &str) -> Result<serde_json::Value> {
        let path = format!("/batchEndpoints/{}", endpoint_name);
        self.get(&path).await
    }

    /// List online endpoints in the workspace
    pub async fn list_online_endpoints(&self) -> Result<serde_json::Value> {
        self.get("/onlineEndpoints").await
    }

    /// Get a specific online endpoint by name
    pub async fn get_online_endpoint(&self, endpoint_name: &str) -> Result<serde_json::Value> {
        let path = format!("/onlineEndpoints/{}", endpoint_name);
        self.get(&path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = AzureMLConfig::default();
        let result = AzureMLClient::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_url_building() {
        let config = AzureMLConfig {
            subscription_id: "sub123".to_string(),
            resource_group_name: "rg123".to_string(),
            workspace_name: "ws123".to_string(),
            ..Default::default()
        };

        let client = AzureMLClient::new(config).unwrap();
        let url = client.build_url("/jobs").unwrap();

        assert!(url.as_str().contains("sub123"));
        assert!(url.as_str().contains("rg123"));
        assert!(url.as_str().contains("ws123"));
        assert!(url.as_str().contains("api-version"));
    }
}
