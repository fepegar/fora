# TextNer

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**primary_metric** | Option<[**models::ClassificationPrimaryMetrics**](ClassificationPrimaryMetrics.md)> |  | [optional]
**featurization_settings** | Option<[**models::NlpVerticalFeaturizationSettings**](NlpVerticalFeaturizationSettings.md)> |  | [optional]
**limit_settings** | Option<[**models::NlpVerticalLimitSettings**](NlpVerticalLimitSettings.md)> |  | [optional]
**validation_data** | Option<[**models::MlTableJobInput**](MLTableJobInput.md)> |  | [optional]
**log_verbosity** | Option<[**models::LogVerbosity**](LogVerbosity.md)> |  | [optional]
**target_column_name** | Option<**String**> | Target column name: This is prediction values column.  Also known as label column name in context of classification tasks. | [optional]
**task_type** | [**models::TaskType**](TaskType.md) |  | 
**training_data** | [**models::MlTableJobInput**](MLTableJobInput.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


