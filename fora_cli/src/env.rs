use anyhow::Result;
use async_trait::async_trait;
use azure_core::http::RequestContent;

use azure_ml::clients::MachineLearningServicesEnvironmentVersionsClient;
use azure_ml::models::AutoRebuildSetting;
use azure_ml::models::BuildContext;
use azure_ml::models::EnvironmentVersion;
use azure_ml::models::EnvironmentVersionProperties;
use azure_ml::models::PendingUploadRequestDto;
use azure_ml::MachineLearningServicesClient;
use azure_storage_blob::*;
use blake3::Hasher;
use futures::future::join_all;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;
use tracing::debug;
use url::Url;
use uuid;

// TODO Do these need to be inside the trait?
// Feels like they could be external functions that take an `impl Environment`
pub async fn create_environment<E: Environment + ?Sized>(
    env: &E,
    ml_client: &MachineLearningServicesClient,
    env_client: &MachineLearningServicesEnvironmentVersionsClient,
    resource_group: &str,
    workspace: &str,
    env_name: &str,
    version: &str,
) -> Result<EnvironmentVersion> {
    let environment_version_body = env
        .get_envirnoment_version_body(ml_client, resource_group, workspace, env_name)
        .await?;

    debug!("Actually creating the environment in Azure ML...");

    let environment_version = env_client
        .create_or_update(
            resource_group,
            workspace,
            env_name,
            version,
            environment_version_body.try_into()?,
            None,
        )
        .await?
        .into_model()?;

    Ok(environment_version)
}

// Same for this function
async fn get_environment(
    env_client: &MachineLearningServicesEnvironmentVersionsClient,
    resource_group: &str,
    workspace: &str,
    env_name: &str,
    version: &str,
) -> Result<EnvironmentVersion> {
    let environment_version = env_client
        .get(&resource_group, &workspace, &env_name, &version, None)
        .await?
        .into_model()?;

    Ok(environment_version)
}

#[async_trait]
pub trait Environment: Sync {
    async fn get_env_name(&self) -> Result<String>;

    fn get_env_version(&self) -> Result<String>;

    async fn get_envirnoment_version_body(
        &self,
        env_client: &MachineLearningServicesClient,
        resource_group: &str,
        workspace: &str,
        env_name: &str,
    ) -> Result<EnvironmentVersion>;

    async fn get_or_create_environment(
        &self,
        ml_client: &MachineLearningServicesClient,
        resource_group: &str,
        workspace: &str,
    ) -> Result<EnvironmentVersion> {
        // Try and get_environment. If it fails, create it

        let env_client = ml_client.get_machine_learning_services_environment_versions_client();
        let env_name = self.get_env_name().await?;
        let version = self.get_env_version()?;

        let environment_version =
            get_environment(&env_client, resource_group, workspace, &env_name, &version).await;

        // match environment_version {
        //     Ok(env) => {
        //         debug!(
        //             "Environment {} version {} already exists. Using existing environment.",
        //             env_name, version
        //         );
        //         // TODO: Check the status. If failed, then maybe try to build again?
        //         return Ok(env);
        //     }
        //     Err(_) => {
        //         debug!(
        //             "Environment {} version {} does not exist. Creating environment.",
        //             env_name, version
        //         );

        // NOTE: Temporarily always try to create a new enironment
        let env = create_environment(
            self,
            ml_client,
            &env_client,
            resource_group,
            workspace,
            &env_name,
            &version,
        )
        .await?;

        debug!(
            "Environment {} version {} created successfully.",
            env_name, version
        );

        return Ok(env);
        //     }
        // };
    }
}

const DOCKERFILE_STR: &str = r#"FROM {base_docker_image}

COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

# Setup a non-root user
RUN groupadd --system --gid 999 nonroot \
 && useradd --system --gid 999 --uid 999 --create-home nonroot

WORKDIR /env
ENV UV_PROJECT_ENVIRONMENT=/env/.venv

# Copy from the cache instead of linking since it's a mounted volume
ENV UV_LINK_MODE=copy

RUN --mount=type=cache,target=/root/.cache/uv \
    --mount=type=bind,source=uv.lock,target=uv.lock \
    --mount=type=bind,source=pyproject.toml,target=pyproject.toml \
    --mount=type=bind,source=.python-version,target=.python-version \
    uv sync --locked --no-install-project

WORKDIR=/workdir

CMD ["bash"]
"#;

const MAX_CONCURRENT_UPLOADS: usize = 10;

// TODO: Can we re-use these structs in the CLI?
// Saves duplicating.
// But how is best to combine base and specific structs
pub struct UvEnv {
    pub auto_rebuild: bool,
    pub base_docker_image: String,
    pub project_dir: PathBuf,
    pub uv_extras: Vec<String>,
    pub uv_groups: Vec<String>,
}

#[async_trait]
impl Environment for UvEnv {
    async fn get_env_name(&self) -> Result<String> {
        // Load pyproject.toml, uv.lock, and .python-version concurrently
        let pyproject_path = self.project_dir.join("pyproject.toml");
        let uv_lock_path = self.project_dir.join("uv.lock");
        let python_version_path = self.project_dir.join(".python-version");

        let (pyproject_result, uv_lock_result, python_version_result) = tokio::join!(
            fs::read_to_string(&pyproject_path),
            fs::read_to_string(&uv_lock_path),
            fs::read_to_string(&python_version_path)
        );

        // Provide specific error messages for missing files
        let pyproject_contents = pyproject_result.map_err(|e| {
            anyhow::anyhow!(
                "Failed to read pyproject.toml at {}: {}",
                pyproject_path.display(),
                e
            )
        })?;

        let uv_lock_contents = uv_lock_result.map_err(|e| {
            anyhow::anyhow!(
                "Failed to read uv.lock at {}: {}",
                uv_lock_path.display(),
                e
            )
        })?;

        // .python-version is optional, so we don't error if it's missing
        let python_version_contents = python_version_result.unwrap_or_else(|_| String::new());

        // Hash the contents using blake3
        let mut hasher = Hasher::new();
        hasher.update(pyproject_contents.as_bytes());
        hasher.update(uv_lock_contents.as_bytes());
        hasher.update(python_version_contents.as_bytes());

        // Include the configuration options in the hash
        hasher.update(self.auto_rebuild.to_string().as_bytes());
        for extra in &self.uv_extras {
            hasher.update(extra.as_bytes());
        }
        for group in &self.uv_groups {
            hasher.update(group.as_bytes());
        }

        let hash = hasher.finalize();

        // Return hash prepended with "Fora-UV-Env-"
        Ok(format!("Fora-UV-Env-{}", hash.to_hex()))
    }

    fn get_env_version(&self) -> Result<String> {
        // TODO: Do properly.
        Ok("tmp".to_string())
    }

    // TODO: We need to get the default datastore so we should pass
    // the ml client instead and create the right clients where needed
    // Function that will be called in the functions defined in Environment
    async fn get_envirnoment_version_body(
        &self,
        ml_client: &MachineLearningServicesClient,
        resource_group: &str,
        workspace: &str,
        env_name: &str,
    ) -> Result<EnvironmentVersion> {
        // Get an upload location (lets try just throwing it in the default datastore in a new fora-env folder)
        // And then create the EnvironmentVersion object

        debug!("Starting environment files upload using SAS token approach...");

        // Upload all files using SAS token approach
        let build_context_uri = self
            .upload_environment_files(ml_client, resource_group, workspace)
            .await?;

        debug!(
            "Environment files uploaded successfully. Build context URI: {}",
            build_context_uri
        );

        let build_context = BuildContext {
            context_uri: Some(build_context_uri),
            ..Default::default()
        };

        let environment_version_properties = EnvironmentVersionProperties {
            auto_rebuild: Some(AutoRebuildSetting::OnBaseImageUpdate),
            build: Some(build_context),
            is_anonymous: Some(true),
            ..Default::default()
        };

        Ok(EnvironmentVersion {
            properties: Some(environment_version_properties),
            ..Default::default()
        })
    }
}

impl UvEnv {
    async fn upload_environment_files(
        &self,
        ml_client: &MachineLearningServicesClient,
        resource_group: &str,
        workspace: &str,
    ) -> Result<String> {
        // Request a SAS URL using the code versions API
        debug!("Requesting SAS URL for environment files upload...");

        let code_client = ml_client.get_machine_learning_services_code_versions_client();
        let container_name = uuid::Uuid::new_v4().to_string();
        let dto = PendingUploadRequestDto {
            ..Default::default()
        };
        let body: RequestContent<PendingUploadRequestDto> = dto.try_into()?;

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

        let blob_reference = pending_upload
            .blob_reference_for_consumption
            .ok_or_else(|| anyhow::anyhow!("Blob reference should be present in pending upload"))?;

        debug!("Got SAS URL for upload: {:?}", blob_reference.credential);

        // Prepare environment files to upload
        let pyproject_path = self.project_dir.join("pyproject.toml");
        let uv_lock_path = self.project_dir.join("uv.lock");
        let python_version_path = self.project_dir.join(".python-version");

        // Parse SAS URI to get upload location
        let upload_credential = blob_reference
            .credential
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No credential in blob reference"))?;

        let sas_uri_str = match upload_credential {
            azure_ml::models::PendingUploadCredentialDto::SASCredentialDto(sas) => sas
                .sas_uri
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("No SAS URI in credential"))?,
            azure_ml::models::PendingUploadCredentialDto::UnknownCredentialType {
                credential_type,
            } => {
                return Err(anyhow::anyhow!(
                    "Unsupported credential type for upload: {:?}",
                    credential_type
                ));
            }
        };

        debug!("Using SAS URI: {}", sas_uri_str);

        let sas_uri = Url::parse(sas_uri_str)?;
        let endpoint_with_sas = format!(
            "{}://{}",
            sas_uri.scheme(),
            sas_uri
                .host_str()
                .ok_or_else(|| anyhow::anyhow!("No host in SAS URI"))?
        );

        let mut path_segments = sas_uri
            .path_segments()
            .ok_or_else(|| anyhow::anyhow!("No path in SAS URI"))?
            .filter(|s| !s.is_empty());

        let container_name = path_segments
            .next()
            .ok_or_else(|| anyhow::anyhow!("No container in SAS URI"))?
            .to_string();

        let sas_token = sas_uri
            .query()
            .ok_or_else(|| anyhow::anyhow!("No SAS token in URI"))?;

        // Read files concurrently
        let (pyproject_result, uv_lock_result, python_version_result) = tokio::join!(
            fs::read_to_string(&pyproject_path),
            fs::read_to_string(&uv_lock_path),
            fs::read_to_string(&python_version_path)
        );

        let pyproject_contents = pyproject_result.map_err(|e| {
            anyhow::anyhow!(
                "Failed to read pyproject.toml at {}: {}",
                pyproject_path.display(),
                e
            )
        })?;

        let uv_lock_contents = uv_lock_result.map_err(|e| {
            anyhow::anyhow!(
                "Failed to read uv.lock at {}: {}",
                uv_lock_path.display(),
                e
            )
        })?;

        // Create Dockerfile with base image replacement
        let dockerfile_contents =
            DOCKERFILE_STR.replace("{base_docker_image}", &self.base_docker_image);

        // Build the list of files to upload, excluding empty python-version
        let mut files_to_upload = vec![
            ("Dockerfile", dockerfile_contents),
            ("pyproject.toml", pyproject_contents),
            ("uv.lock", uv_lock_contents),
        ];

        // Only add .python-version if it's not empty or if the file exists
        if let Ok(python_version_contents) = python_version_result {
            if !python_version_contents.trim().is_empty() {
                files_to_upload.push((".python-version", python_version_contents));
            } else {
                debug!(".python-version file is empty, skipping upload");
            }
        } else {
            debug!(".python-version file not found, skipping upload");
        }

        debug!("Will upload {} files", files_to_upload.len());

        // Prepare upload tasks using SAS token
        let endpoint_with_sas = Arc::new(format!("{}?{}", endpoint_with_sas, sas_token));
        let container_name = Arc::new(container_name);
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_UPLOADS));

        debug!(
            "Starting upload of environment files with up to {} concurrent uploads using SAS...",
            MAX_CONCURRENT_UPLOADS
        );

        let tasks: Vec<_> = files_to_upload
            .into_iter()
            .map(|(filename, content)| {
                let endpoint_with_sas = Arc::clone(&endpoint_with_sas);
                let container_name = Arc::clone(&container_name);
                let semaphore = Arc::clone(&semaphore);
                let filename = filename.to_string(); // Clone filename for logging

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();

                    debug!("Starting upload of file: {}", filename);

                    let blob_name = filename.clone();
                    let data = content.into_bytes();
                    let content_length = data.len() as u64;

                    debug!(
                        "Uploading {} ({} bytes) as blob: {}",
                        filename, content_length, blob_name
                    );

                    let request_content = RequestContent::from(data);

                    // Use SAS token - no credential needed
                    let blob_client = BlobClient::new(
                        &endpoint_with_sas,
                        &container_name,
                        &blob_name,
                        None::<Arc<dyn azure_core::credentials::TokenCredential>>,
                        Some(BlobClientOptions::default()),
                    )?;

                    match blob_client
                        .upload(
                            request_content,
                            true, // overwrite
                            content_length,
                            None, // upload options
                        )
                        .await
                    {
                        Ok(_) => {
                            debug!("Successfully uploaded: {} as {}", filename, blob_name);
                            Ok(blob_name)
                        }
                        Err(e) => {
                            debug!("Failed to upload {}: {}", filename, e);
                            Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                        }
                    }
                })
            })
            .collect();

        debug!("Spawned {} upload tasks", tasks.len());

        let results = join_all(tasks).await;

        // Better error handling and logging
        let mut failed_uploads = Vec::new();
        let mut successful_uploads = Vec::new();

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(inner_result) => match inner_result {
                    Ok(blob_name) => {
                        successful_uploads.push(blob_name);
                    }
                    Err(e) => {
                        failed_uploads.push(format!("Upload task {}: {}", i, e));
                    }
                },
                Err(join_err) => {
                    failed_uploads.push(format!("Task {} panicked: {}", i, join_err));
                }
            }
        }

        debug!(
            "Upload results: {} successful, {} failed",
            successful_uploads.len(),
            failed_uploads.len()
        );

        if !failed_uploads.is_empty() {
            for failure in &failed_uploads {
                debug!("Upload failure: {}", failure);
            }
            return Err(anyhow::anyhow!("{} uploads failed", failed_uploads.len()));
        }

        debug!(
            "Successfully uploaded all environment files: {:?}",
            successful_uploads
        );

        // Return the blob URI for the build context
        let blob_uri = blob_reference
            .blob_uri
            .ok_or_else(|| anyhow::anyhow!("No blob URI in reference"))?;

        // blob uri is like https://{name}.blob.core.windows.net:443/{container}
        // We need to remove the port
        let blob_uri = blob_uri.replace(":443", "");

        Ok(blob_uri)
    }
}

// fn hash_folder_contents() {}

// pub struct DockerEnv {
//     docker_content_path: PathBuf,
//     dockerfile: Option<PathBuf>,
// }

// #[async_trait]
// impl Environment for DockerEnv {
//     async fn get_env_name(&self) -> Result<String> {
//         // Hash the contents of the folder to create a unique name
//         let folder_hash = hash_folder_contents(&self.docker_content_path)?;
//         let env_name = &fodler_hash[..32];
//         let env_name = format!("DockerEnv-{}", env_name);
//         Ok(env_name)
//     }

//     fn get_env_version(&self) -> Result<String> {
//         Ok("latest".to_string())
//     }

//     // Function that will be called in the functions defined in Environment
// }
