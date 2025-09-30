# BatchDeployment

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**compute** | Option<**String**> | Compute target for batch inference operation. | [optional]
**deployment_configuration** | Option<[**models::BatchDeploymentConfiguration**](BatchDeploymentConfiguration.md)> |  | [optional]
**error_threshold** | Option<**i32**> | Error threshold, if the error count for the entire input goes above this value,  the batch inference will be aborted. Range is [-1, int.MaxValue].  For FileDataset, this value is the count of file failures.  For TabularDataset, this value is the count of record failures.  If set to -1 (the lower bound), all failures during batch inference will be ignored. | [optional][default to -1]
**logging_level** | Option<[**models::BatchLoggingLevel**](BatchLoggingLevel.md)> |  | [optional]
**max_concurrency_per_instance** | Option<**i32**> | Indicates maximum number of parallelism per instance. | [optional][default to 1]
**mini_batch_size** | Option<**i64**> | Size of the mini-batch passed to each batch invocation.  For FileDataset, this is the number of files per mini-batch.  For TabularDataset, this is the size of the records in bytes, per mini-batch. | [optional][default to 10]
**model** | Option<[**models::AssetReferenceBase**](AssetReferenceBase.md)> |  | [optional]
**output_action** | Option<[**models::BatchOutputAction**](BatchOutputAction.md)> |  | [optional]
**output_file_name** | Option<**String**> | Customized output file name for append_row output action. | [optional][default to predictions.csv]
**provisioning_state** | Option<[**models::DeploymentProvisioningState**](DeploymentProvisioningState.md)> |  | [optional]
**resources** | Option<[**models::DeploymentResourceConfiguration**](DeploymentResourceConfiguration.md)> |  | [optional]
**retry_settings** | Option<[**models::BatchRetrySettings**](BatchRetrySettings.md)> |  | [optional]
**code_configuration** | Option<[**models::CodeConfiguration**](CodeConfiguration.md)> |  | [optional]
**description** | Option<**String**> | Description of the endpoint deployment. | [optional]
**environment_id** | Option<**String**> | ARM resource ID or AssetId of the environment specification for the endpoint deployment. | [optional]
**environment_variables** | Option<**std::collections::HashMap<String, String>**> | Environment variables configuration for the deployment. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | Property dictionary. Properties can be added, but not removed or altered. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


