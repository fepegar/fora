# TableVertical

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cv_split_column_names** | Option<**Vec<String>**> | Columns to use for CVSplit data. | [optional]
**featurization_settings** | Option<[**models::TableVerticalFeaturizationSettings**](TableVerticalFeaturizationSettings.md)> |  | [optional]
**limit_settings** | Option<[**models::TableVerticalLimitSettings**](TableVerticalLimitSettings.md)> |  | [optional]
**n_cross_validations** | Option<[**models::NCrossValidations**](NCrossValidations.md)> |  | [optional]
**test_data** | Option<[**models::MlTableJobInput**](MLTableJobInput.md)> |  | [optional]
**test_data_size** | Option<**f64**> | The fraction of test dataset that needs to be set aside for validation purpose.  Values between (0.0 , 1.0)  Applied when validation dataset is not provided. | [optional]
**validation_data** | Option<[**models::MlTableJobInput**](MLTableJobInput.md)> |  | [optional]
**validation_data_size** | Option<**f64**> | The fraction of training dataset that needs to be set aside for validation purpose.  Values between (0.0 , 1.0)  Applied when validation dataset is not provided. | [optional]
**weight_column_name** | Option<**String**> | The name of the sample weight column. Automated ML supports a weighted column as an input, causing rows in the data to be weighted up or down. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


