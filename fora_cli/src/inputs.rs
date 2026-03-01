use anyhow::Result;
use regex::Regex;
use std::backtrace::Backtrace;
use std::str::FromStr;
use thiserror::Error;

use azure_ml::clients::MachineLearningServicesDataVersionsClient;
use azure_ml::models::{
    DataVersionBaseProperties, InputDeliveryMode, JobInput, MLTableJobInput, UriFileJobInput,
    UriFolderJobInput,
};

pub trait AzureMLInput {
    async fn get_rest_input(
        &self,
        client: MachineLearningServicesDataVersionsClient,
        resource_group_name: &str,
        workspace_name: &str,
    ) -> Result<JobInput>;

    fn get_alias(&self) -> &str;
}

#[derive(Clone, Debug)]
pub struct DataAssetInput {
    pub alias: String,
    pub data_asset_id: String,
    pub version: String,
    pub mode: InputDeliveryMode,
}

impl AzureMLInput for DataAssetInput {
    async fn get_rest_input(
        &self,
        client: MachineLearningServicesDataVersionsClient,
        resource_group_name: &str,
        workspace_name: &str,
    ) -> Result<JobInput> {
        // TODO: Probably don't use `?`. I can't remember
        let data_version = client
            .get(
                &resource_group_name,
                &workspace_name,
                &self.data_asset_id,
                &self.version,
                None,
            )
            .await?
            .into_model()?;
        let properties = data_version.properties.unwrap();
        let description = None;
        let mode = Some(self.mode.clone());
        let job_input = match properties {
            DataVersionBaseProperties::MLTableData(value) => {
                JobInput::MLTableJobInput(MLTableJobInput {
                    description,
                    mode,
                    uri: value.data_uri,
                })
            }
            DataVersionBaseProperties::UriFileDataVersion(value) => {
                JobInput::UriFileJobInput(UriFileJobInput {
                    description,
                    mode,
                    uri: value.data_uri,
                })
            }
            DataVersionBaseProperties::UriFolderDataVersion(value) => {
                JobInput::UriFolderJobInput(UriFolderJobInput {
                    description,
                    mode,
                    uri: value.data_uri,
                })
            }
            DataVersionBaseProperties::UnknownDataType { data_uri, .. } => {
                JobInput::UriFolderJobInput(UriFolderJobInput {
                    description,
                    mode,
                    uri: data_uri,
                })
            }
        };
        Ok(job_input)
    }

    fn get_alias(&self) -> &str {
        &self.alias
    }
}

fn parse_data_asset_string(input: &str) -> Option<(String, String, Option<String>)> {
    let re = Regex::new(r"^([^=]+)=([^:]+)(?::(.+))?$").unwrap();

    if let Some(captures) = re.captures(input) {
        let alias = captures.get(1)?.as_str().trim().to_string();
        let asset = captures.get(2)?.as_str().trim().to_string();
        let version = captures.get(3).map(|m| m.as_str().trim().to_string());

        if alias.is_empty() || asset.is_empty() {
            return None;
        }

        Some((alias, asset, version))
    } else {
        None
    }
}

#[derive(Error, Debug)]
#[error("Failed to parse input string: {msg}")]
pub struct InputParseError {
    msg: String,
}

impl FromStr for DataAssetInput {
    type Err = InputParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Support alias=asset:version
        if let Some((alias, asset, version)) = parse_data_asset_string(s) {
            Ok(Self {
                alias,
                data_asset_id: asset,
                version: version.unwrap_or_else(|| "latest".to_string()),
                mode: InputDeliveryMode::ReadOnlyMount, // Default mode
            })
        } else {
            Err(InputParseError {
                msg: format!(
                    "Invalid input format: '{}'. Expected 'alias=asset:version'",
                    s
                ),
            })
        }
    }
}

#[derive(Clone, Debug)]
pub struct DatastoreInput {
    pub alias: String,
    pub datastore_name: String,
    pub path: String,
    pub mode: InputDeliveryMode,
}

impl AzureMLInput for DatastoreInput {
    async fn get_rest_input(
        &self,
        _client: MachineLearningServicesDataVersionsClient,
        _resource_group_name: &str,
        _workspace_name: &str,
    ) -> Result<JobInput> {
        // For datastore inputs, we can construct the URI directly
        let uri = format!(
            "azureml://datastores/{}/paths/{}",
            self.datastore_name, self.path
        );
        let job_input = UriFolderJobInput {
            description: None,
            mode: Some(self.mode.clone()),
            uri: Some(uri),
        };
        Ok(JobInput::UriFolderJobInput(job_input))
    }

    fn get_alias(&self) -> &str {
        &self.alias
    }
}

// TODO: Implement FromStr for InputEnum to auto get the correct
// input type?
// Or rather than enum, can functions take impl AzureMLInput? That
// would be nicer if possible.
// A simple function `parse_input_string` that returns a Box<dyn AzureMLInput>
// might be good. And use dynamic dispatch
#[derive(Clone, Debug)]
pub enum InputEnum {
    DataAssetInput(DataAssetInput),
    DatastoreInput(DatastoreInput),
}

impl AzureMLInput for InputEnum {
    async fn get_rest_input(
        &self,
        client: MachineLearningServicesDataVersionsClient,
        resource_group_name: &str,
        workspace_name: &str,
    ) -> Result<JobInput> {
        match self {
            InputEnum::DataAssetInput(data_asset_input) => {
                data_asset_input
                    .get_rest_input(client, resource_group_name, workspace_name)
                    .await
            }
            InputEnum::DatastoreInput(datastore_input) => {
                datastore_input
                    .get_rest_input(client, resource_group_name, workspace_name)
                    .await
            }
        }
    }

    fn get_alias(&self) -> &str {
        match self {
            InputEnum::DataAssetInput(data_asset_input) => data_asset_input.get_alias(),
            InputEnum::DatastoreInput(datastore_input) => datastore_input.get_alias(),
        }
    }
}

impl FromStr for InputEnum {
    type Err = InputParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Not implemented, return an error for now
        Err(InputParseError {
            msg: format!("Parsing not implemented for input string: '{}'", s),
        })
    }
}
