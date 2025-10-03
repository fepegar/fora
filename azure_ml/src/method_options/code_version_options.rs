use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct CodeVersionsCreateOrGetStartPendingUploadOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct CodeVersionsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct CodeVersionsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct CodeVersionsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct CodeVersionsListOptions<'a> {
    pub dollar_order_by: Option<String>,
    pub dollar_top: Option<i32>,
    pub dollar_skip: Option<String>,
    pub hash: Option<String>,
    pub hash_version: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl CodeVersionsListOptions<'_> {
    pub fn into_owned(self) -> CodeVersionsListOptions<'static> {
        CodeVersionsListOptions {
            dollar_order_by: self.dollar_order_by,
            dollar_top: self.dollar_top,
            dollar_skip: self.dollar_skip,
            hash: self.hash,
            hash_version: self.hash_version,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

#[derive(Clone, Default, SafeDebug)]
pub struct CodeVersionsPublishOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryCodeVersionsCreateOrGetStartPendingUploadOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryCodeVersionsCreateOrUpdateOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryCodeVersionsDeleteOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryCodeVersionsGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct RegistryCodeVersionsListOptions<'a> {
    pub dollar_order_by: Option<String>,
    pub dollar_top: Option<i32>,
    pub dollar_skip: Option<String>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl RegistryCodeVersionsListOptions<'_> {
    pub fn into_owned(self) -> RegistryCodeVersionsListOptions<'static> {
        RegistryCodeVersionsListOptions {
            dollar_order_by: self.dollar_order_by,
            dollar_top: self.dollar_top,
            dollar_skip: self.dollar_skip,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

