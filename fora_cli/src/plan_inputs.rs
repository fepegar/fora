pub trait AzureMLInput {
    async fn get_rest_input(&self, ml_client);
    fn get_alias(&self) -> String;
}

#[derive(Clone)]
pub struct UriInput {
    pub alias: String,
    pub uri: String,
    pub mode: UriInputMode,
    pub input_type: String,
}

impl AzureMLInput for UriInput {
    async fn get_rest_input(&self, ml_client) {
        // Fill Contents
    }

    fn get_alias(&self) -> String {
        &self.alias
    }
}

impl FromString for UriInput {
    fn from_string(s: &str) -> Result<Self> {

    }
}

#[derive(Clone)]
pub struct DataAssetInput {
    pub alias: String,
    pub data_asset_id: String,
    pub version: String,
    pub mode: String,
}

impl AzureMLInput for DataAssetInput {
    async fn get_rest_input(&self, ml_client) {
        // Fill Contents
    }

    fn get_alias(&self) -> String {
        &self.alias
    }
}

impl FromString for DataAssetInput {
    fn from_string(s: &str) -> Result<Self> {
        // Support alias=asset:version
    }
}


#[derive(Clone)]
pub enum InputEnum {
    UriInput(UriInput),
    DataAssetInput(DataAssetInput),
}

impl AzureMLInput for InputEnum {
    async fn get_rest_input(&self, ml_client) {
        match self {
            InputEnum::UriInput(uri_input) => uri_input.get_rest_input(ml_client).await,
            InputEnum::DataAssetInput(data_asset_input) => data_asset_input.get_rest_input(ml_client).await,
        }
    }

    fn get_alias(&self) -> String {
        match self {
            InputEnum::UriInput(uri_input) => uri_input.get_alias(),
            InputEnum::DataAssetInput(data_asset_input) => data_asset_input.get_alias(),
        }
    }
}
