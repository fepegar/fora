use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineEndpointsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineEndpointsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineEndpointsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineEndpointsGetTokenOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineEndpointsListOptions<'a> {
    pub name: Option<String>,
    pub count: Option<i32>,
    pub compute_type: Option<String>,
    pub dollar_skip: Option<String>,
    pub tags: Option<String>,
    pub properties: Option<String>,
    pub order_by: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl OnlineEndpointsListOptions<'_> {
    pub fn into_owned(self) -> OnlineEndpointsListOptions<'static> {
        OnlineEndpointsListOptions {
            name: self.name,
            count: self.count,
            compute_type: self.compute_type,
            dollar_skip: self.dollar_skip,
            tags: self.tags,
            properties: self.properties,
            order_by: self.order_by,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineEndpointsListKeysOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineEndpointsRegenerateKeysOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineEndpointsUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}
