

#[async_trait]
pub trait Environment: Sync {
    async fn get_env_name(&self) -> Result<String>;

    fn get_env_version(&self) -> Result<String>;

    async fn create_environment(
        &self,
        ml_client,
        env_name,
        version: Option<String>,
    ) {
        // Fill Contents
    }

    async fn get_environment(
        &self,
        ml_client,
        env_name,
        version: Option<String>,
    ) -> Result<EnvironmentResource> {
        // Fill Contents
    }

    async fn get_or_create_environment(
        &self,
        ml_client,
        env_name,
        version: Option<String>,
    ) -> Result<EnvironmentResource> {
        // Fill contents
    }
}

fn hash_folder_contents() {

}

pub struct DockerEnv {
    docker_content_path: PathBuf,
    dockerfile: Option<PathBuf>,
}

#[async_trait]
impl Environment for DockerEnv {
    async fn get_env_name(&self) -> Result<String> {
        // Hash the contents of the folder to create a unique name
        let folder_hash = hash_folder_contents(&self.docker_content_path)?;
        let env_name = &fodler_hash[..32];
        let env_name = format!("DockerEnv-{}", env_name);
        Ok(env_name)
    }

    fn get_env_version(&self) -> Result<String> {
        Ok("latest".to_string())
    }

    // Function that will be called in the functions defined in Environment
}

pub struct UvEnv {
    pyproject_path: PathBuf,
    base_docker_image: String,
}

#[async_trait]
impl Environment for UvEnv {
    async fn get_env_name(&self) -> Result<String> {
        // Hash the contents of the pyproject.toml, uv.lock, etc. to create a unique name
    }

}
