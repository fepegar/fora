# FeatureAttributionDriftMonitoringSignal

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**feature_data_type_override** | Option<[**std::collections::HashMap<String, models::MonitoringFeatureDataType>**](MonitoringFeatureDataType.md)> | A dictionary that maps feature names to their respective data types. | [optional]
**feature_importance_settings** | [**models::FeatureImportanceSettings**](FeatureImportanceSettings.md) |  | 
**metric_threshold** | [**models::FeatureAttributionMetricThreshold**](FeatureAttributionMetricThreshold.md) |  | 
**production_data** | [**Vec<models::MonitoringInputDataBase>**](MonitoringInputDataBase.md) | [Required] The data which drift will be calculated for. | 
**reference_data** | [**models::MonitoringInputDataBase**](MonitoringInputDataBase.md) |  | 
**notification_types** | Option<[**Vec<models::MonitoringNotificationType>**](MonitoringNotificationType.md)> | The current notification mode for this signal. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | Property dictionary. Properties can be added, but not removed or altered. | [optional]
**signal_type** | [**models::MonitoringSignalType**](MonitoringSignalType.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


