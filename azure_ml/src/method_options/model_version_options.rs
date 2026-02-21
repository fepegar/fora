use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct ModelVersionsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ModelVersionsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ModelVersionsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ModelVersionsListOptions<'a> {
    pub dollar_skip: Option<String>,
    pub dollar_order_by: Option<String>,
    pub dollar_top: Option<i32>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub offset: Option<i32>,
    pub tags: Option<String>,
    pub properties: Option<String>,
    pub feed: Option<String>,
    pub list_view_type: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl ModelVersionsListOptions<'_> {
    pub fn into_owned(self) -> ModelVersionsListOptions<'static> {
        ModelVersionsListOptions {
            dollar_skip: self.dollar_skip,
            dollar_order_by: self.dollar_order_by,
            dollar_top: self.dollar_top,
            version: self.version,
            description: self.description,
            offset: self.offset,
            tags: self.tags,
            properties: self.properties,
            feed: self.feed,
            list_view_type: self.list_view_type,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

#[derive(Clone, Default, SafeDebug)]
pub struct ModelVersionsPublishOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryModelVersionsCreateOrGetStartPendingUploadOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryModelVersionsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryModelVersionsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryModelVersionsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryModelVersionsListOptions<'a> {
    pub dollar_skip: Option<String>,
    pub dollar_order_by: Option<String>,
    pub dollar_top: Option<i32>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub properties: Option<String>,
    pub list_view_type: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl RegistryModelVersionsListOptions<'_> {
    pub fn into_owned(self) -> RegistryModelVersionsListOptions<'static> {
        RegistryModelVersionsListOptions {
            dollar_skip: self.dollar_skip,
            dollar_order_by: self.dollar_order_by,
            dollar_top: self.dollar_top,
            version: self.version,
            description: self.description,
            tags: self.tags,
            properties: self.properties,
            list_view_type: self.list_view_type,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}
