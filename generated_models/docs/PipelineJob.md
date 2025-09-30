# PipelineJob

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**inputs** | Option<[**std::collections::HashMap<String, models::JobInput>**](JobInput.md)> | Inputs for the pipeline job. | [optional]
**jobs** | Option<[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)> | Jobs construct the Pipeline Job. | [optional]
**outputs** | Option<[**std::collections::HashMap<String, models::JobOutput>**](JobOutput.md)> | Outputs for the pipeline job | [optional]
**settings** | Option<[**serde_json::Value**](.md)> | Pipeline settings, for things like ContinueRunOnStepFailure etc. | [optional]
**source_job_id** | Option<**String**> | ARM resource ID of source job. | [optional]
**component_id** | Option<**String**> | ARM resource ID of the component resource. | [optional]
**compute_id** | Option<**String**> | ARM resource ID of the compute resource. | [optional]
**display_name** | Option<**String**> | Display name of job. | [optional]
**experiment_name** | Option<**String**> | The name of the experiment the job belongs to. If not set, the job is placed in the \"Default\" experiment. | [optional][default to Default]
**identity** | Option<[**models::IdentityConfiguration**](IdentityConfiguration.md)> |  | [optional]
**is_archived** | Option<**bool**> | Is the asset archived? | [optional][default to false]
**job_type** | [**models::JobType**](JobType.md) |  | 
**notification_setting** | Option<[**models::NotificationSetting**](NotificationSetting.md)> |  | [optional]
**services** | Option<[**std::collections::HashMap<String, models::JobService>**](JobService.md)> | List of JobEndpoints.  For local jobs, a job endpoint will have an endpoint value of FileStreamObject. | [optional]
**status** | Option<[**models::JobStatus**](JobStatus.md)> |  | [optional]
**description** | Option<**String**> | The asset description text. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | The asset property dictionary. | [optional]
**tags** | Option<**std::collections::HashMap<String, String>**> | Tag dictionary. Tags can be added, removed, and updated. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


