use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineDeploymentsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineDeploymentsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineDeploymentsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineDeploymentsGetLogsOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineDeploymentsListOptions<'a> {
    pub dollar_order_by: Option<String>,
    pub dollar_top: Option<i32>,
    pub dollar_skip: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl OnlineDeploymentsListOptions<'_> {
    pub fn into_owned(self) -> OnlineDeploymentsListOptions<'static> {
        OnlineDeploymentsListOptions {
            dollar_order_by: self.dollar_order_by,
            dollar_top: self.dollar_top,
            dollar_skip: self.dollar_skip,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineDeploymentsListSkusOptions<'a> {
    pub count: Option<i32>,
    pub dollar_skip: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl OnlineDeploymentsListSkusOptions<'_> {
    pub fn into_owned(self) -> OnlineDeploymentsListSkusOptions<'static> {
        OnlineDeploymentsListSkusOptions {
            count: self.count,
            dollar_skip: self.dollar_skip,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

#[derive(Clone, Default, SafeDebug)]
pub struct OnlineDeploymentsUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

