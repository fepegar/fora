# Classification

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**positive_label** | Option<**String**> | Positive label for binary metrics calculation. | [optional]
**primary_metric** | Option<[**models::ClassificationPrimaryMetrics**](ClassificationPrimaryMetrics.md)> |  | [optional]
**training_settings** | Option<[**models::ClassificationTrainingSettings**](ClassificationTrainingSettings.md)> |  | [optional]
**cv_split_column_names** | Option<**Vec<String>**> | Columns to use for CVSplit data. | [optional]
**featurization_settings** | Option<[**models::TableVerticalFeaturizationSettings**](TableVerticalFeaturizationSettings.md)> |  | [optional]
**limit_settings** | Option<[**models::TableVerticalLimitSettings**](TableVerticalLimitSettings.md)> |  | [optional]
**n_cross_validations** | Option<[**models::NCrossValidations**](NCrossValidations.md)> |  | [optional]
**test_data** | Option<[**models::MlTableJobInput**](MLTableJobInput.md)> |  | [optional]
**test_data_size** | Option<**f64**> | The fraction of test dataset that needs to be set aside for validation purpose.  Values between (0.0 , 1.0)  Applied when validation dataset is not provided. | [optional]
**validation_data** | Option<[**models::MlTableJobInput**](MLTableJobInput.md)> |  | [optional]
**validation_data_size** | Option<**f64**> | The fraction of training dataset that needs to be set aside for validation purpose.  Values between (0.0 , 1.0)  Applied when validation dataset is not provided. | [optional]
**weight_column_name** | Option<**String**> | The name of the sample weight column. Automated ML supports a weighted column as an input, causing rows in the data to be weighted up or down. | [optional]
**log_verbosity** | Option<[**models::LogVerbosity**](LogVerbosity.md)> |  | [optional]
**target_column_name** | Option<**String**> | Target column name: This is prediction values column.  Also known as label column name in context of classification tasks. | [optional]
**task_type** | [**models::TaskType**](TaskType.md) |  | 
**training_data** | [**models::MlTableJobInput**](MLTableJobInput.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


