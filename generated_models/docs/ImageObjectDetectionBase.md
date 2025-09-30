# ImageObjectDetectionBase

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**model_settings** | Option<[**models::ImageModelSettingsObjectDetection**](ImageModelSettingsObjectDetection.md)> |  | [optional]
**search_space** | Option<[**Vec<models::ImageModelDistributionSettingsObjectDetection>**](ImageModelDistributionSettingsObjectDetection.md)> | Search space for sampling different combinations of models and their hyperparameters. | [optional]
**limit_settings** | [**models::ImageLimitSettings**](ImageLimitSettings.md) |  | 
**sweep_settings** | Option<[**models::ImageSweepSettings**](ImageSweepSettings.md)> |  | [optional]
**validation_data** | Option<[**models::MlTableJobInput**](MLTableJobInput.md)> |  | [optional]
**validation_data_size** | Option<**f64**> | The fraction of training dataset that needs to be set aside for validation purpose.  Values between (0.0 , 1.0)  Applied when validation dataset is not provided. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


