# TableVerticalFeaturizationSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**blocked_transformers** | Option<[**Vec<models::BlockedTransformers>**](BlockedTransformers.md)> | These transformers shall not be used in featurization. | [optional]
**column_name_and_types** | Option<**std::collections::HashMap<String, String>**> | Dictionary of column name and its type (int, float, string, datetime etc). | [optional]
**enable_dnn_featurization** | Option<**bool**> | Determines whether to use Dnn based featurizers for data featurization. | [optional][default to false]
**mode** | Option<[**models::FeaturizationMode**](FeaturizationMode.md)> |  | [optional]
**transformer_params** | Option<[**std::collections::HashMap<String, Vec<models::ColumnTransformer>>**](Vec.md)> | User can specify additional transformers to be used along with the columns to which it would be applied and parameters for the transformer constructor. | [optional]
**dataset_language** | Option<**String**> | Dataset language, useful for the text data. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


