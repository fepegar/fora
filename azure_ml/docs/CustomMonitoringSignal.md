# CustomMonitoringSignal

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**component_id** | **String** | [Required] Reference to the component asset used to calculate the custom metrics. | 
**input_assets** | Option<[**std::collections::HashMap<String, models::MonitoringInputDataBase>**](MonitoringInputDataBase.md)> | Monitoring assets to take as input. Key is the component input port name, value is the data asset. | [optional]
**inputs** | Option<[**std::collections::HashMap<String, models::JobInput>**](JobInput.md)> | Extra component parameters to take as input. Key is the component literal input port name, value is the parameter value. | [optional]
**metric_thresholds** | [**Vec<models::CustomMetricThreshold>**](CustomMetricThreshold.md) | [Required] A list of metrics to calculate and their associated thresholds. | 
**notification_types** | Option<[**Vec<models::MonitoringNotificationType>**](MonitoringNotificationType.md)> | The current notification mode for this signal. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | Property dictionary. Properties can be added, but not removed or altered. | [optional]
**signal_type** | [**models::MonitoringSignalType**](MonitoringSignalType.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


