use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct ComponentContainersCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ComponentContainersDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ComponentContainersGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ComponentContainersListOptions<'a> {
    pub dollar_skip: Option<String>,
    pub list_view_type: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl ComponentContainersListOptions<'_> {
    pub fn into_owned(self) -> ComponentContainersListOptions<'static> {
        ComponentContainersListOptions {
            dollar_skip: self.dollar_skip,
            list_view_type: self.list_view_type,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryComponentContainersCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryComponentContainersDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryComponentContainersGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryComponentContainersListOptions<'a> {
    pub dollar_skip: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl RegistryComponentContainersListOptions<'_> {
    pub fn into_owned(self) -> RegistryComponentContainersListOptions<'static> {
        RegistryComponentContainersListOptions {
            dollar_skip: self.dollar_skip,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

