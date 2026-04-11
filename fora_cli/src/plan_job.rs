
pub trait ComputeTarget {
    fn get_arm_resource_id(&self) -> String;
}

pub struct AzureMLComputeTarget {
    pub worksapce: Workspace,
    pub cluster: String
}

impl ComputeTarget for AzureMLComputeTarget {
    fn get_arm_resource_id(&self) -> String {
        format!(
            "/subscriptions/{}/resourceGroups/{}/providers/Microsoft.MachineLearningServices/workspaces/{}/computes/{}",
            self.worksapce.subscription_id,
            self.worksapce.resource_group,
            self.worksapce.name,
            self.cluster
        )
    }
}

pub trait CloudInfo {
    fn get_job_resource_builder(&self) -> JobResourceBuilder {
        JobResourceBuilder::default().to_owned();
    }

    fn get_command_job_builder(&self) -> CommandJobBuilder {
        CommandJobBuilder::default().to_owned();
    }

    fn get_compute_target(&self) -> &dyn ComputeTarget;
}

pub struct AzureMLInfo {
    pub compute_target: AzureMLComputeTarget,
}


pub struct Job {
    pub display_name: Option<String>,
    pub experiment_name: Option<String>,
    pub distribution: Distribution,
    pub environment_variables: Option<HashMap<String, String>>,

    // etc

    pub environment: Box<dyn Environment>,
    pub cloud_info: Box<dyn CloudInfo>,
}

impl Job {

    fn get_command_job_builder(&self) -> CommandJobBuilder {

        let compute_id = self.cloud_info
            .get_compute_target()
            .get_arm_resource_id();

        let builder = builder
            .command(self.command)
            .display_name(self.display_name.clone())
            .experiment_name(self.experiment_name.clone())
            .environment_variables(self.environment_variables.clone());
        // etc
    }

    pub async fn submit_job_async(&self, ml_client) {
        // Like in other file
    }

    pub fn submit_job(&self, ml_client) {
        // Call submit_job_async and block on it
        let rt = Runtime::new().unwrap();
        let output = rt.block_on(self.submit_job_async(ml_client));
        Ok(output)
    }
}
