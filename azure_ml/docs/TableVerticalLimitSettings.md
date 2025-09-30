# TableVerticalLimitSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**enable_early_termination** | Option<**bool**> | Enable early termination, determines whether or not if AutoMLJob will terminate early if there is no score improvement in last 20 iterations. | [optional][default to true]
**exit_score** | Option<**f64**> | Exit score for the AutoML job. | [optional]
**max_concurrent_trials** | Option<**i32**> | Maximum Concurrent iterations. | [optional][default to 1]
**max_cores_per_trial** | Option<**i32**> | Max cores per iteration. | [optional][default to -1]
**max_trials** | Option<**i32**> | Number of iterations. | [optional][default to 1000]
**timeout** | Option<**String**> | AutoML job timeout. | [optional][default to PT6H]
**trial_timeout** | Option<**String**> | Iteration timeout. | [optional][default to PT30M]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


