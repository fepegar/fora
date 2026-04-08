use std::sync::Arc;

use azure_core::{
    credentials::TokenCredential,
    error::CheckSuccessOptions,
    http::{
        policies::{auth::BearerTokenAuthorizationPolicy, Policy},
        ClientOptions, Method, Pipeline, PipelineSendOptions, Request, Url, UrlExt,
    },
    json, Result,
};

use super::models::*;

const ARM_ENDPOINT: &str = "https://management.azure.com";
const GRAPH_ENDPOINT: &str = "https://graph.microsoft.com";
const ARM_SCOPE: &str = "https://management.azure.com/.default";
const GRAPH_SCOPE: &str = "https://graph.microsoft.com/.default";
const ARM_SUBSCRIPTION_API_VERSION: &str = "2024-03-01";


/// Client for discovering Azure resources (subscriptions, resource groups, ML workspaces)
/// and fetching the current user's profile from Microsoft Graph.
pub struct DiscoveryClient {
    arm_pipeline: Pipeline,
    graph_pipeline: Pipeline,
}

impl DiscoveryClient {
    /// Creates a new `DiscoveryClient` using the provided credential.
    pub fn new(credential: Arc<dyn TokenCredential>) -> Self {
        let arm_auth: Arc<dyn Policy> = Arc::new(BearerTokenAuthorizationPolicy::new(
            credential.clone(),
            vec![ARM_SCOPE],
        ));
        let graph_auth: Arc<dyn Policy> = Arc::new(BearerTokenAuthorizationPolicy::new(
            credential,
            vec![GRAPH_SCOPE],
        ));

        let arm_pipeline = Pipeline::new(
            option_env!("CARGO_PKG_NAME"),
            option_env!("CARGO_PKG_VERSION"),
            ClientOptions::default(),
            Vec::default(),
            vec![arm_auth],
            None,
        );

        let graph_pipeline = Pipeline::new(
            option_env!("CARGO_PKG_NAME"),
            option_env!("CARGO_PKG_VERSION"),
            ClientOptions::default(),
            Vec::default(),
            vec![graph_auth],
            None,
        );

        Self {
            arm_pipeline,
            graph_pipeline,
        }
    }

    /// Fetches the current user's profile from Microsoft Graph.
    pub async fn get_current_user(&self) -> Result<UserProfile> {
        let url: Url = format!("{}/v1.0/me", GRAPH_ENDPOINT).parse()?;
        let mut request = Request::new(url, Method::Get);
        request.insert_header("accept", "application/json");

        let ctx = azure_core::http::ClientMethodOptions::default();
        let rsp = self
            .graph_pipeline
            .send(
                &ctx.context,
                &mut request,
                Some(PipelineSendOptions {
                    check_success: CheckSuccessOptions {
                        success_codes: &[200],
                    },
                    ..Default::default()
                }),
            )
            .await?;

        let (_status, _headers, body) = rsp.deconstruct();
        let profile: UserProfile = json::from_json(&body)?;
        Ok(profile)
    }

    /// Lists all Azure subscriptions accessible to the current user.
    pub async fn list_subscriptions(&self) -> Result<Vec<Subscription>> {
        let mut all = Vec::new();
        let mut url = self.arm_url("/subscriptions", ARM_SUBSCRIPTION_API_VERSION)?;

        loop {
            let result: SubscriptionListResult = self.arm_get(&url).await?;
            all.extend(result.value);

            match result.next_link {
                Some(next_link) if !next_link.is_empty() => {
                    url = next_link.parse()?;
                }
                _ => break,
            }
        }

        Ok(all)
    }

    fn arm_url(&self, path: &str, api_version: &str) -> Result<Url> {
        let mut url: Url = ARM_ENDPOINT.parse()?;
        url.append_path(path);
        let mut qb = url.query_builder();
        qb.set_pair("api-version", api_version);
        qb.build();
        Ok(url)
    }

    async fn arm_get<T: serde::de::DeserializeOwned>(&self, url: &Url) -> Result<T> {
        let mut request = Request::new(url.clone(), Method::Get);
        request.insert_header("accept", "application/json");

        let ctx = azure_core::http::ClientMethodOptions::default();
        let rsp = self
            .arm_pipeline
            .send(
                &ctx.context,
                &mut request,
                Some(PipelineSendOptions {
                    check_success: CheckSuccessOptions {
                        success_codes: &[200],
                    },
                    ..Default::default()
                }),
            )
            .await?;

        let (_status, _headers, body) = rsp.deconstruct();
        let result: T = json::from_json(&body)?;
        Ok(result)
    }
}
