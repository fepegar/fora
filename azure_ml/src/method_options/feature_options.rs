use azure_core::{fmt::SafeDebug, http::ClientMethodOptions};

#[derive(Clone, Default, SafeDebug)]
pub struct FeaturesGetOptions<'a> {
    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

#[derive(Clone, Default, SafeDebug)]
pub struct FeaturesListOptions<'a> {
    pub dollar_skip: Option<String>,
    pub tags: Option<String>,
    pub feature_name: Option<String>,
    pub description: Option<String>,
    pub list_view_type: Option<String>,
    pub page_size: Option<i32>,

    /// Allows customization of the method call.
    pub method_options: ClientMethodOptions<'a>,
}

impl FeaturesListOptions<'_> {
    pub fn into_owned(self) -> FeaturesListOptions<'static> {
        FeaturesListOptions {
            dollar_skip: self.dollar_skip,
            tags: self.tags,
            feature_name: self.feature_name,
            description: self.description,
            list_view_type: self.list_view_type,
            page_size: self.page_size,
            method_options: ClientMethodOptions {
                context: self.method_options.context.into_owned(),
            },
        }
    }
}

