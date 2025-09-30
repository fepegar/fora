# SparkJob

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**archives** | Option<**Vec<String>**> | Archive files used in the job. | [optional]
**args** | Option<**String**> | Arguments for the job. | [optional]
**code_id** | **String** | [Required] arm-id of the code asset. | 
**conf** | Option<**std::collections::HashMap<String, String>**> | Spark configured properties. | [optional]
**entry** | [**models::SparkJobEntry**](SparkJobEntry.md) |  | 
**environment_id** | Option<**String**> | The ARM resource ID of the Environment specification for the job. | [optional]
**environment_variables** | Option<**std::collections::HashMap<String, String>**> | Environment variables included in the job. | [optional]
**files** | Option<**Vec<String>**> | Files used in the job. | [optional]
**inputs** | Option<[**std::collections::HashMap<String, models::JobInput>**](JobInput.md)> | Mapping of input data bindings used in the job. | [optional]
**jars** | Option<**Vec<String>**> | Jar files used in the job. | [optional]
**outputs** | Option<[**std::collections::HashMap<String, models::JobOutput>**](JobOutput.md)> | Mapping of output data bindings used in the job. | [optional]
**py_files** | Option<**Vec<String>**> | Python files used in the job. | [optional]
**queue_settings** | Option<[**models::QueueSettings**](QueueSettings.md)> |  | [optional]
**resources** | Option<[**models::SparkResourceConfiguration**](SparkResourceConfiguration.md)> |  | [optional]
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


