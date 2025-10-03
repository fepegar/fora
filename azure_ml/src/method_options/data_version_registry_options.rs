use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryDataVersionsCreateOrGetStartPendingUploadOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryDataVersionsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryDataVersionsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryDataVersionsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryDataVersionsListOptions<'a> {
    pub dollar_order_by: Option<String>,
    pub dollar_top: Option<i32>,
    pub dollar_skip: Option<String>,
    pub dollar_tags: Option<String>,
    pub list_view_type: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl RegistryDataVersionsListOptions<'_> {
    pub fn into_owned(self) -> RegistryDataVersionsListOptions<'static> {
        RegistryDataVersionsListOptions {
            dollar_order_by: self.dollar_order_by,
            dollar_top: self.dollar_top,
            dollar_skip: self.dollar_skip,
            dollar_tags: self.dollar_tags,
            list_view_type: self.list_view_type,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

