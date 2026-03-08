use anyhow::anyhow;
use anyhow::Result;
use azure_identity::AzureCliCredential;
use azure_ml::models::CodeVersion;
use azure_ml::models::CommandJob;
use azure_ml::models::EnvironmentVersion;
use azure_ml::models::JobBase;
use azure_ml::models::JobBaseProperties;
use azure_ml::MachineLearningServicesClient;

use crate::cli::EnvironmentType;
use crate::cli::SubmitArgs;
use crate::code::upload_folder_to_generated_location;
use crate::env::Environment;
use crate::env::UvEnv;

pub async fn submit_to_azure(args: &SubmitArgs) -> Result<()> {
    // Upload code
    let resource_group = &args.resource_group;
    let workspace = &args.workspace;
    let subscription = &args.subscription;

    // Create Azure credential
    let credential = AzureCliCredential::new(None)?;

    // Create the Machine Learning Services client
    let ml_client = MachineLearningServicesClient::new(
        "https://management.azure.com",
        credential,
        subscription.clone(),
        None,
    )?;

    // pub auto_rebuild: bool,
    // pub base_docker_image: String,
    // pub project_dir: PathBuf,
    // pub uv_extras: Vec<String>,
    // pub uv_groups: Vec<String>,

    let default_docker_image = "mcr.microsoft.com/azureml/openmpi5.0-cuda12.6-ubuntu24.04";
    let current_dir = std::env::current_dir()?;
    let source_folder = args.source.as_ref().unwrap_or(&current_dir);

    let env = match args.env_args.env_type {
        EnvironmentType::Uv => {
            let env = UvEnv {
                auto_rebuild: args.env_args.auto_rebuild,
                base_docker_image: args
                    .env_args
                    .base_docker_image
                    .clone()
                    .unwrap_or(default_docker_image.to_string()),
                project_dir: args
                    .uv_args
                    .project_dir
                    .clone()
                    .unwrap_or(source_folder.clone()),
                uv_extras: args.uv_args.uv_extra.clone(),
                uv_groups: args.uv_args.uv_group.clone(),
            };
            Ok(env)
        }
        _ => Err(anyhow!("Not implemented")),
    }?;

    let code_version_future =
        upload_folder_to_generated_location(&ml_client, resource_group, workspace, source_folder);

    let env_future = env.get_or_create_environment(&ml_client, resource_group, workspace);

    let (code_version_result, env_version_result) = futures::join!(code_version_future, env_future);

    let code_version = code_version_result?;
    let environment_version = env_version_result?;

    Ok(())
}

async fn submit_job(
    ml_client: &MachineLearningServicesClient,
    resouce_group: &str,
    workspace: &str,
    code_version: &CodeVersion,
    environment_version: &EnvironmentVersion,
    args: &SubmitArgs,
) -> Result<()> {
    let jobs_client = ml_client.get_machine_learning_services_jobs_client();

    let run_id = names::Generator::default()
        .next()
        .unwrap()
        .to_string()
        .to_lowercase();

    let compute_id = format!(
        "/subscriptions/{}/resourceGroups/{}/providers/Microsoft.MachineLearningServices/workspaces/{}/computes/{}",
        args.subscription,
        resouce_group,
        workspace,
        &args.cluster,
    );

    let command = format!(
        "{} {} {}",
        args.command_prefix.unwrap_or("".to_string()),
        args.executable.unwrap_or("python".to_string()),
        args.cmd.join(" ")
    );

    let command_job = CommandJob {
        code_id: Some(code_version.id.unwrap()),
        command: Some(command),
        compute_id: Some(compute_id),
        display_name: args.name,
        // distribution: None, // TODO: Support distribution
        environment_id: Some(environment_version.id.unwrap()),
        environment_variables: None, // TODO: Support environment variables
        experiment_name: Some(args.experiment.clone()),
        // inputs: None, // TODO: Support inputs
        // outputs: None, // TODO: Support outputs
        // resources: None, // TODO: Support resource requirements - used for num_nodes, etc.
        ..Default::default()
    };

    let job_properties = JobBaseProperties::CommandJob(command_job);

    let job_body = JobBase {
        properties: Some(job_properties),
        ..Default::default()
    };

    let _job_result = jobs_client
        .create_or_update(
            resouce_group,
            workspace,
            &run_id,
            job_body.try_into()?,
            None,
        )
        .await?
        .into_model();

    Ok(())
}
