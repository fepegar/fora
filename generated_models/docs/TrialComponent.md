# TrialComponent

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**code_id** | Option<**String**> | ARM resource ID of the code asset. | [optional]
**command** | **String** | [Required] The command to execute on startup of the job. eg. \"python train.py\" | 
**distribution** | Option<[**models::DistributionConfiguration**](DistributionConfiguration.md)> |  | [optional]
**environment_id** | **String** | [Required] The ARM resource ID of the Environment specification for the job. | 
**environment_variables** | Option<**std::collections::HashMap<String, String>**> | Environment variables included in the job. | [optional]
**resources** | Option<[**models::JobResourceConfiguration**](JobResourceConfiguration.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


