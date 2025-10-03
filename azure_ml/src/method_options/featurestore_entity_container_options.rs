use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct FeaturestoreEntityContainersCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct FeaturestoreEntityContainersDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct FeaturestoreEntityContainersGetEntityOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct FeaturestoreEntityContainersListOptions<'a> {
    pub dollar_skip: Option<String>,
    pub tags: Option<String>,
    pub list_view_type: Option<String>,
    pub page_size: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl FeaturestoreEntityContainersListOptions<'_> {
    pub fn into_owned(self) -> FeaturestoreEntityContainersListOptions<'static> {
        FeaturestoreEntityContainersListOptions {
            dollar_skip: self.dollar_skip,
            tags: self.tags,
            list_view_type: self.list_view_type,
            page_size: self.page_size,
            name: self.name,
            description: self.description,
            created_by: self.created_by,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

