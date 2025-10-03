use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct FeaturesetVersionsBackfillOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct FeaturesetVersionsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct FeaturesetVersionsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct FeaturesetVersionsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct FeaturesetVersionsListOptions<'a> {
    pub dollar_skip: Option<String>,
    pub tags: Option<String>,
    pub list_view_type: Option<String>,
    pub page_size: Option<i32>,
    pub version_name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub stage: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl FeaturesetVersionsListOptions<'_> {
    pub fn into_owned(self) -> FeaturesetVersionsListOptions<'static> {
        FeaturesetVersionsListOptions {
            dollar_skip: self.dollar_skip,
            tags: self.tags,
            list_view_type: self.list_view_type,
            page_size: self.page_size,
            version_name: self.version_name,
            version: self.version,
            description: self.description,
            created_by: self.created_by,
            stage: self.stage,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

