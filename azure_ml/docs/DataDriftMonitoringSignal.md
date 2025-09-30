# DataDriftMonitoringSignal

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**feature_data_type_override** | Option<[**std::collections::HashMap<String, models::MonitoringFeatureDataType>**](MonitoringFeatureDataType.md)> | A dictionary that maps feature names to their respective data types. | [optional]
**feature_importance_settings** | Option<[**models::FeatureImportanceSettings**](FeatureImportanceSettings.md)> |  | [optional]
**features** | Option<[**models::MonitoringFeatureFilterBase**](MonitoringFeatureFilterBase.md)> |  | [optional]
**metric_thresholds** | [**Vec<models::DataDriftMetricThresholdBase>**](DataDriftMetricThresholdBase.md) | [Required] A list of metrics to calculate and their associated thresholds. | 
**production_data** | [**models::MonitoringInputDataBase**](MonitoringInputDataBase.md) |  | 
**reference_data** | [**models::MonitoringInputDataBase**](MonitoringInputDataBase.md) |  | 
**notification_types** | Option<[**Vec<models::MonitoringNotificationType>**](MonitoringNotificationType.md)> | The current notification mode for this signal. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | Property dictionary. Properties can be added, but not removed or altered. | [optional]
**signal_type** | [**models::MonitoringSignalType**](MonitoringSignalType.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


