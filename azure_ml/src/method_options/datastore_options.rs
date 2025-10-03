use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct DatastoresCreateOrUpdateOptions<'a> {
    pub skip_validation: Option<bool>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl DatastoresCreateOrUpdateOptions<'_> {
    pub fn into_owned(self) -> DatastoresCreateOrUpdateOptions<'static> {
        DatastoresCreateOrUpdateOptions {
            skip_validation: self.skip_validation,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

#[derive(Clone, Default, SafeDebug)]
pub struct DatastoresDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct DatastoresGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct DatastoresListOptions<'a> {
    pub dollar_skip: Option<String>,
    pub count: Option<i32>,
    pub is_default: Option<bool>,
    pub names: Option<Vec<String>>,
    pub search_text: Option<String>,
    pub order_by: Option<String>,
    pub order_by_asc: Option<bool>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl DatastoresListOptions<'_> {
    pub fn into_owned(self) -> DatastoresListOptions<'static> {
        DatastoresListOptions {
            dollar_skip: self.dollar_skip,
            count: self.count,
            is_default: self.is_default,
            names: self.names,
            search_text: self.search_text,
            order_by: self.order_by,
            order_by_asc: self.order_by_asc,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

#[derive(Clone, Default, SafeDebug)]
pub struct DatastoresListSecretsOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

