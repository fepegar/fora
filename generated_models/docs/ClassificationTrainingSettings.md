# ClassificationTrainingSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**allowed_training_algorithms** | Option<[**Vec<models::ClassificationModels>**](ClassificationModels.md)> | Allowed models for classification task. | [optional]
**blocked_training_algorithms** | Option<[**Vec<models::ClassificationModels>**](ClassificationModels.md)> | Blocked models for classification task. | [optional]
**enable_dnn_training** | Option<**bool**> | Enable recommendation of DNN models. | [optional][default to false]
**enable_model_explainability** | Option<**bool**> | Flag to turn on explainability on best model. | [optional][default to true]
**enable_onnx_compatible_models** | Option<**bool**> | Flag for enabling onnx compatible models. | [optional][default to false]
**enable_stack_ensemble** | Option<**bool**> | Enable stack ensemble run. | [optional][default to true]
**enable_vote_ensemble** | Option<**bool**> | Enable voting ensemble run. | [optional][default to true]
**ensemble_model_download_timeout** | Option<**String**> | During VotingEnsemble and StackEnsemble model generation, multiple fitted models from the previous child runs are downloaded.  Configure this parameter with a higher value than 300 secs, if more time is needed. | [optional][default to PT5M]
**stack_ensemble_settings** | Option<[**models::StackEnsembleSettings**](StackEnsembleSettings.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


