# StackEnsembleSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**stack_meta_learner_k_wargs** | Option<[**serde_json::Value**](.md)> | Optional parameters to pass to the initializer of the meta-learner. | [optional]
**stack_meta_learner_train_percentage** | Option<**f64**> | Specifies the proportion of the training set (when choosing train and validation type of training) to be reserved for training the meta-learner. Default value is 0.2. | [optional][default to 0.2]
**stack_meta_learner_type** | Option<[**models::StackMetaLearnerType**](StackMetaLearnerType.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


