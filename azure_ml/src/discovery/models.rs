use serde::Deserialize;

/// User profile from Microsoft Graph API.
#[derive(Debug, Clone, Deserialize)]
pub struct UserProfile {
    #[serde(rename = "displayName")]
    pub display_name: String,
}

/// Azure subscription.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Subscription {
    #[serde(rename = "subscriptionId")]
    pub subscription_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

/// Response wrapper for subscription list.
#[derive(Debug, Deserialize)]
pub(crate) struct SubscriptionListResult {
    pub value: Vec<Subscription>,
    #[serde(rename = "nextLink")]
    pub next_link: Option<String>,
}

/// Azure resource group.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ResourceGroup {
    pub name: String,
    pub location: String,
}

/// Response wrapper for resource group list.
#[derive(Debug, Deserialize)]
pub(crate) struct ResourceGroupListResult {
    pub value: Vec<ResourceGroup>,
    #[serde(rename = "nextLink")]
    pub next_link: Option<String>,
}

/// Azure Machine Learning workspace.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MlWorkspace {
    pub name: String,
    pub location: String,
}

/// Response wrapper for ML workspace list.
#[derive(Debug, Deserialize)]
pub(crate) struct MlWorkspaceListResult {
    pub value: Vec<MlWorkspace>,
    #[serde(rename = "nextLink")]
    pub next_link: Option<String>,
}
