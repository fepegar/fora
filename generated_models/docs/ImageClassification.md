# ImageClassification

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**primary_metric** | Option<[**models::ClassificationPrimaryMetrics**](ClassificationPrimaryMetrics.md)> |  | [optional]
**model_settings** | Option<[**models::ImageModelSettingsClassification**](ImageModelSettingsClassification.md)> |  | [optional]
**search_space** | Option<[**Vec<models::ImageModelDistributionSettingsClassification>**](ImageModelDistributionSettingsClassification.md)> | Search space for sampling different combinations of models and their hyperparameters. | [optional]
**limit_settings** | [**models::ImageLimitSettings**](ImageLimitSettings.md) |  | 
**sweep_settings** | Option<[**models::ImageSweepSettings**](ImageSweepSettings.md)> |  | [optional]
**validation_data** | Option<[**models::MlTableJobInput**](MLTableJobInput.md)> |  | [optional]
**validation_data_size** | Option<**f64**> | The fraction of training dataset that needs to be set aside for validation purpose.  Values between (0.0 , 1.0)  Applied when validation dataset is not provided. | [optional]
**log_verbosity** | Option<[**models::LogVerbosity**](LogVerbosity.md)> |  | [optional]
**target_column_name** | Option<**String**> | Target column name: This is prediction values column.  Also known as label column name in context of classification tasks. | [optional]
**task_type** | [**models::TaskType**](TaskType.md) |  | 
**training_data** | [**models::MlTableJobInput**](MLTableJobInput.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


