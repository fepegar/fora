use serde::Deserialize;

/// User profile from Microsoft Graph API.
#[derive(Debug, Clone, Deserialize)]
pub struct UserProfile {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "givenName")]
    pub given_name: String,
    pub surname: String,
}

impl UserProfile {
    /// Returns the user's full name as "{givenName} {surname}".
    pub fn name(&self) -> String {
        format!("{} {}", self.given_name, self.surname)
    }
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
