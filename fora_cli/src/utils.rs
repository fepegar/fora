use anyhow::Result;
use azure_ml::models::AzureBlobDatastore;
use azure_ml::models::DatastoreProperties;
use azure_ml::models::MachineLearningServicesDatastoresClientListOptions;
use azure_ml::MachineLearningServicesClient;
use futures::TryStreamExt as _;

// TODO: This should take as input the datastores client so it's not
// constructed multiple times unnecesssarily
// TODO: We should have a cache in the ~/.fora directory
// So we don't have to call this every time. Unless it's very fast.
pub async fn get_default_container(
    ml_client: &MachineLearningServicesClient,
    resource_group: &str,
    workspace: &str,
) -> Result<AzureBlobDatastore> {
    let datastores_client = ml_client.get_machine_learning_services_datastores_client();

    let options = MachineLearningServicesDatastoresClientListOptions {
        is_default: Some(true),
        ..Default::default()
    };

    let mut datastores_pager = datastores_client.list(resource_group, workspace, Some(options))?;
    // Loop over until we find one with `is_default` set to true. There should only be one, but we'll just take the first one we find.
    // If we don't find any, we'll return an error.
    while let Some(datastore) = datastores_pager.try_next().await? {
        let datastore_properties = datastore
            .properties
            .expect("Default datastore should have properties")
            .clone();

        match datastore_properties {
            DatastoreProperties::AzureBlobDatastore(d) => {
                let is_default = d.is_default.unwrap_or(false);
                if is_default {
                    return Ok(d);
                } else {
                    continue; // Not the default datastore, keep looking
                }
            }
            _ => return Err(anyhow::anyhow!("Default datastore is not a blob datastore")),
        };
    }
    Err(anyhow::anyhow!("No default datastore found in workspace"))
}
