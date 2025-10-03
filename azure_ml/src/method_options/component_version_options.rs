use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct ComponentVersionsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ComponentVersionsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ComponentVersionsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ComponentVersionsListOptions<'a> {
    pub dollar_order_by: Option<String>,
    pub dollar_top: Option<i32>,
    pub dollar_skip: Option<String>,
    pub list_view_type: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl ComponentVersionsListOptions<'_> {
    pub fn into_owned(self) -> ComponentVersionsListOptions<'static> {
        ComponentVersionsListOptions {
            dollar_order_by: self.dollar_order_by,
            dollar_top: self.dollar_top,
            dollar_skip: self.dollar_skip,
            list_view_type: self.list_view_type,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

#[derive(Clone, Default, SafeDebug)]
pub struct ComponentVersionsPublishOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryComponentVersionsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryComponentVersionsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryComponentVersionsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryComponentVersionsListOptions<'a> {
    pub dollar_order_by: Option<String>,
    pub dollar_top: Option<i32>,
    pub dollar_skip: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl RegistryComponentVersionsListOptions<'_> {
    pub fn into_owned(self) -> RegistryComponentVersionsListOptions<'static> {
        RegistryComponentVersionsListOptions {
            dollar_order_by: self.dollar_order_by,
            dollar_top: self.dollar_top,
            dollar_skip: self.dollar_skip,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

