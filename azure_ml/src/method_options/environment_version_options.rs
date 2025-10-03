use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct EnvironmentVersionsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct EnvironmentVersionsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct EnvironmentVersionsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct EnvironmentVersionsListOptions<'a> {
    pub dollar_order_by: Option<String>,
    pub dollar_top: Option<i32>,
    pub dollar_skip: Option<String>,
    pub list_view_type: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl EnvironmentVersionsListOptions<'_> {
    pub fn into_owned(self) -> EnvironmentVersionsListOptions<'static> {
        EnvironmentVersionsListOptions {
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
pub struct EnvironmentVersionsPublishOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryEnvironmentVersionsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryEnvironmentVersionsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryEnvironmentVersionsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryEnvironmentVersionsListOptions<'a> {
    pub dollar_order_by: Option<String>,
    pub dollar_top: Option<i32>,
    pub dollar_skip: Option<String>,
    pub list_view_type: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl RegistryEnvironmentVersionsListOptions<'_> {
    pub fn into_owned(self) -> RegistryEnvironmentVersionsListOptions<'static> {
        RegistryEnvironmentVersionsListOptions {
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

