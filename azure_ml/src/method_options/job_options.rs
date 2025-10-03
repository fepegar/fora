use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct JobsCancelOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct JobsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct JobsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct JobsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct JobsListOptions<'a> {
    pub dollar_skip: Option<String>,
    pub job_type: Option<String>,
    pub tag: Option<String>,
    pub list_view_type: Option<String>,
    pub properties: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl JobsListOptions<'_> {
    pub fn into_owned(self) -> JobsListOptions<'static> {
        JobsListOptions {
            dollar_skip: self.dollar_skip,
            job_type: self.job_type,
            tag: self.tag,
            list_view_type: self.list_view_type,
            properties: self.properties,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}
