# RollingInputData

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**preprocessing_component_id** | Option<**String**> | Reference to the component asset used to preprocess the data. | [optional]
**window_offset** | **String** | [Required] The time offset between the end of the data window and the monitor's current run time. | 
**window_size** | **String** | [Required] The size of the rolling data window. | 
**columns** | Option<**std::collections::HashMap<String, String>**> | Mapping of column names to special uses. | [optional]
**data_context** | Option<**String**> | The context metadata of the data source. | [optional]
**input_data_type** | [**models::MonitoringInputDataType**](MonitoringInputDataType.md) |  | 
**job_input_type** | [**models::JobInputType**](JobInputType.md) |  | 
**uri** | **String** | [Required] Input Asset URI. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


