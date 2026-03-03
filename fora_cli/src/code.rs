use anyhow::Result;
use azure_core::http::RequestContent;
use azure_ml::models::BlobReferenceForConsumptionDto;
use azure_ml::models::CodeVersion;
use azure_ml::models::CodeVersionProperties;
use azure_ml::models::PendingUploadCredentialDto;
use azure_ml::models::PendingUploadRequestDto;
use azure_ml::MachineLearningServicesClient;
use azure_storage_blob::*;
use futures::future::join_all;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;
use tracing::debug;
use tracing::info;
use url::Url;

const MAX_CONCURRENT_UPLOADS: usize = 128;

pub async fn upload_folder_to_generated_location(
    ml_client: &MachineLearningServicesClient,
    resource_group: &str,
    workspace: &str,
    source_folder: &Path,
) -> Result<CodeVersion> {
    let container_name = uuid::Uuid::new_v4().to_string();
    debug!("Using container '{}' for code upload", container_name);
    let code_client = ml_client.get_machine_learning_services_code_versions_client();
    let dto = PendingUploadRequestDto {
        ..Default::default()
    };
    let body: RequestContent<PendingUploadRequestDto> = dto.try_into()?;
    debug!("Requesting pending upload from Azure ML...");
    // NOTE: The container name doesn't appear to actually be used
    // It creates a random container
    let pending_upload = code_client
        .create_or_get_start_pending_upload(
            resource_group,
            workspace,
            &container_name,
            "latest",
            body,
            None,
        )
        .await?
        .into_model()?;

    // Source defaults to current directory if not specified
    let blob_reference = pending_upload
        .blob_reference_for_consumption
        .expect("Blob reference should be present in pending upload");

    debug!("Uploading code...");
    upload_folder_to_pending_upload_location(&source_folder, &blob_reference).await?;

    // TODO: Do unwrapping more safely
    let code_version_properties = CodeVersionProperties {
        code_uri: Some(blob_reference.blob_uri.unwrap()),
        is_anonymous: Some(true),
        ..Default::default()
    };
    let code_version_request = CodeVersion {
        properties: Some(code_version_properties),
        ..Default::default()
    };
    let code_version_request_content: RequestContent<CodeVersion> =
        code_version_request.try_into()?;

    debug!("Creating code version from uploaded code...");
    let code_version = code_client
        .create_or_update(
            &resource_group,
            &workspace,
            &container_name,
            "1", // Must be a positive integer as a string
            code_version_request_content,
            None,
        )
        .await?
        .into_model()?;

    let code_version_id = code_version
        .id
        .as_ref()
        .map(|id| id.as_str())
        .unwrap_or("No ID");

    debug!("Code version created with ID: {}", code_version_id);

    Ok(code_version)
}

pub async fn upload_folder_to_pending_upload_location(
    source_folder: &Path,
    location: &BlobReferenceForConsumptionDto,
) -> Result<()> {
    // Parse the SAS URI to extract the components we need
    // let blob_uri = &location.blob_reference_for_consumption.blob_uri;
    let upload_credential = &location.credential.as_ref().unwrap();

    // TODO: Can we just use our own credential?
    // I.e. use blob_uri rather than credential.
    let sas_uri_str = match upload_credential {
        PendingUploadCredentialDto::SASCredentialDto(value) => value
            .sas_uri
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("SAS URI is None"))?,
        PendingUploadCredentialDto::UnknownCredentialType { credential_type } => {
            return Err(anyhow::anyhow!(
                "Unsupported credential type for upload: {:?}",
                credential_type
            ));
        }
    };

    debug!("Using SAS URI: {}", sas_uri_str);

    let sas_uri = Url::parse(sas_uri_str)?;

    // Extract storage account endpoint: "https://account.blob.core.windows.net/"
    let endpoint = format!(
        "{}://{}/",
        sas_uri.scheme(),
        sas_uri
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("No host in SAS URI"))?
    );

    // SAS token is the query string on the SAS URI — embed it in the endpoint
    // The new crate accepts the SAS token baked into the endpoint URL, with no credential needed.
    let sas_token = sas_uri
        .query()
        .ok_or_else(|| anyhow::anyhow!("No SAS token in SAS URI"))?;
    let endpoint_with_sas = format!("{}?{}", endpoint.trim_end_matches('/'), sas_token);

    // Parse container name and blob prefix from blob_uri path: /<container>/<prefix...>
    let mut path_segments = sas_uri
        .path_segments()
        .ok_or_else(|| anyhow::anyhow!("No path in blob URI"))?
        .filter(|s| !s.is_empty());

    let container_name = path_segments
        .next()
        .ok_or_else(|| anyhow::anyhow!("No container in blob URI"))?
        .to_string();

    let blob_prefix: String = path_segments.collect::<Vec<_>>().join("/");
    let blob_prefix = if blob_prefix.is_empty() {
        String::new()
    } else {
        format!("{}/", blob_prefix)
    };

    // Wrap in Arc so it's cheaply cloneable across tasks
    let endpoint_with_sas = Arc::new(endpoint_with_sas);
    let container_name = Arc::new(container_name);
    let blob_prefix = Arc::new(blob_prefix);

    let files = collect_files(source_folder);
    info!("Uploading {} files", files.len());

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_UPLOADS));

    let tasks: Vec<_> = files
        .into_iter()
        .map(|file_path| {
            let endpoint_with_sas = Arc::clone(&endpoint_with_sas);
            let container_name = Arc::clone(&container_name);
            let blob_prefix = Arc::clone(&blob_prefix);
            let semaphore = Arc::clone(&semaphore);
            let source_folder = source_folder.to_path_buf();

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                let relative = file_path
                    .strip_prefix(&source_folder)
                    .expect("file must be under source folder");

                let blob_name = format!(
                    "{}{}",
                    blob_prefix,
                    relative.to_string_lossy().replace('\\', "/")
                );

                let data = fs::read(&file_path).await?;
                let content_length = data.len() as u64;
                let content = RequestContent::from(data);

                // Each BlobClient is per-blob, so we create one per task.
                // The SAS token is embedded in the endpoint URL; credential is None.
                let blob_client = BlobClient::new(
                    &endpoint_with_sas,
                    &container_name,
                    &blob_name,
                    None::<Arc<dyn azure_core::credentials::TokenCredential>>,
                    Some(BlobClientOptions::default()),
                )?;

                blob_client
                    .upload(
                        content,
                        true, // overwrite
                        content_length,
                        None, // upload options
                    )
                    .await?;

                debug!("Uploaded: {}", blob_name);
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(blob_name)
            })
        })
        .collect();

    let results = join_all(tasks).await;

    let failures = results
        .iter()
        .filter(|r| r.as_ref().map_or(true, |inner| inner.is_err()))
        .count();

    println!(
        "Done: {} uploaded, {} failed",
        results.len() - failures,
        failures
    );

    if failures > 0 {
        return Err(anyhow::anyhow!("{} uploads failed", failures));
    }

    Ok(())
}

async fn zip_folder_and_upload() {}

fn collect_files(folder: &Path) -> Vec<PathBuf> {
    WalkBuilder::new(folder)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .add_custom_ignore_filename(".amlignore")
        .build()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                Some(entry.into_path())
            } else {
                None
            }
        })
        .collect()
}
