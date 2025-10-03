use crate::method_options::JobsListOptions;
use crate::models::JobBaseResourceArmPaginatedResult;
use azure_core::{
    credentials::TokenCredential,
    fmt::SafeDebug,
    http::{
        check_success,
        pager::{PagerResult, PagerState},
        policies::{BearerTokenCredentialPolicy, Policy},
        BufResponse, ClientOptions, Method, Pager, Pipeline, Request, Url,
    },
    json, tracing, Result,
};
use std::sync::Arc;

macro_rules! append_query_param {
    ($url:expr, $param_name:expr, $value:expr) => {
        if let Some(val) = $value {
            $url.query_pairs_mut().append_pair($param_name, &val);
        }
    };
}

#[tracing::client]
pub struct MLClient {
    pub(crate) api_version: String,
    pub(crate) base_url: Url,
    pub(crate) pipeline: Pipeline,
}

#[derive(Clone, SafeDebug)]
pub struct MLClientOptions {
    /// ClientOptions for customizing the pipeline.
    pub client_options: ClientOptions,
}

impl MLClient {
    /// Creates a new MLClient, using Entra ID authentication.
    #[tracing::new("MachineLearning")]
    pub fn new(
        subscription_id: impl Into<String>,
        resource_group_name: impl Into<String>,
        workspace_name: impl Into<String>,
        credential: Arc<dyn TokenCredential>,
        options: Option<MLClientOptions>,
    ) -> Result<Self> {
        let subscription_id = subscription_id.into();
        let resource_group_name = resource_group_name.into();
        let workspace_name = workspace_name.into();

        let options = options.unwrap_or_default();
        let auth_policy: Arc<dyn Policy> = Arc::new(BearerTokenCredentialPolicy::new(
            credential,
            vec!["https://management.azure.com/.default"],
        ));

        // Construct the workspace base URL once
        let mut base_url = Url::parse("https://management.azure.com/")?;
        let path = format!(
            "subscriptions/{}/resourceGroups/{}/providers/Microsoft.MachineLearningServices/workspaces/{}/",
            subscription_id, resource_group_name, workspace_name
        );
        base_url = base_url.join(&path)?;

        Ok(Self {
            api_version: String::from("2025-09-01"),
            base_url,
            pipeline: Pipeline::new(
                option_env!("CARGO_PKG_NAME"),
                option_env!("CARGO_PKG_VERSION"),
                options.client_options,
                Vec::default(),
                vec![auth_policy],
                None,
            ),
        })
    }

    /// Lists jobs in the configured workspace.
    ///
    /// GET {base_url}/jobs?api-version=2025-09-01
    #[tracing::function("MachineLearning.listJobs")]
    pub fn list_jobs(
        &self,
        options: Option<JobsListOptions<'_>>,
    ) -> Result<Pager<JobBaseResourceArmPaginatedResult>> {
        let options = options.unwrap_or_default().into_owned();
        let pipeline = self.pipeline.clone();

        // start URL from base_url
        let mut first_url = self.base_url.join("jobs")?;
        first_url
            .query_pairs_mut()
            .append_pair("api-version", &self.api_version);

        // TODO: Is there a better way to do this?
        append_query_param!(first_url, "$skip", options.dollar_skip);
        append_query_param!(first_url, "job_type", options.job_type);
        append_query_param!(first_url, "tag", options.tag);
        append_query_param!(first_url, "list_view_type", options.list_view_type);
        append_query_param!(first_url, "properties", options.properties);

        let api_version = self.api_version.clone();
        Ok(Pager::from_callback(move |next_link: PagerState<Url>| {
            let url = match next_link {
                PagerState::More(next_link) => {
                    let qp = next_link
                        .query_pairs()
                        .filter(|(name, _)| name.ne("api-version"));
                    let mut next_link = next_link.clone();
                    next_link
                        .query_pairs_mut()
                        .clear()
                        .extend_pairs(qp)
                        .append_pair("api-version", &api_version);
                    next_link
                }
                PagerState::Initial => first_url.clone(),
            };
            let mut request = Request::new(url, Method::Get);
            request.insert_header("accept", "application/json");
            let ctx = options.method_options.context.clone();
            let pipeline = pipeline.clone();
            async move {
                let rsp = pipeline.send(&ctx, &mut request).await?;
                let rsp = check_success(rsp).await?;
                let (status, headers, body) = rsp.deconstruct();
                let bytes = body.collect().await?;
                let res: JobBaseResourceArmPaginatedResult = json::from_json(&bytes)?;
                let rsp = BufResponse::from_bytes(status, headers, bytes).into();
                Ok(match res.next_link {
                    Some(next_link) if !next_link.is_empty() => PagerResult::More {
                        response: rsp,
                        continuation: next_link.parse()?,
                    },
                    _ => PagerResult::Done { response: rsp },
                })
            }
        }))
    }
}

impl Default for MLClientOptions {
    fn default() -> Self {
        Self {
            client_options: ClientOptions::default(),
        }
    }
}
