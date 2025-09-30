# DataCollector

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**collections** | [**std::collections::HashMap<String, models::Collection>**](Collection.md) | [Required] The collection configuration. Each collection has it own configuration to collect model data and the name of collection can be arbitrary string.  Model data collector can be used for either payload logging or custom logging or both of them. Collection request and response are reserved for payload logging, others are for custom logging. | 
**request_logging** | Option<[**models::RequestLogging**](RequestLogging.md)> |  | [optional]
**rolling_rate** | Option<[**models::RollingRateType**](RollingRateType.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


