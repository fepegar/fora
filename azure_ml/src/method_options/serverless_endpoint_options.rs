use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct ServerlessEndpointsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ServerlessEndpointsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ServerlessEndpointsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ServerlessEndpointsListOptions<'a> {
    pub dollar_skip: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl ServerlessEndpointsListOptions<'_> {
    pub fn into_owned(self) -> ServerlessEndpointsListOptions<'static> {
        ServerlessEndpointsListOptions {
            dollar_skip: self.dollar_skip,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

#[derive(Clone, Default, SafeDebug)]
pub struct ServerlessEndpointsListKeysOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ServerlessEndpointsRegenerateKeysOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct ServerlessEndpointsUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

