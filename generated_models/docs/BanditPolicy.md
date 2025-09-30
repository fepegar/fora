# BanditPolicy

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**slack_amount** | Option<**f32**> | Absolute distance allowed from the best performing run. | [optional][default to 0.0]
**slack_factor** | Option<**f32**> | Ratio of the allowed distance from the best performing run. | [optional][default to 0.0]
**delay_evaluation** | Option<**i32**> | Number of intervals by which to delay the first evaluation. | [optional][default to 0]
**evaluation_interval** | Option<**i32**> | Interval (number of runs) between policy evaluations. | [optional][default to 0]
**policy_type** | [**models::EarlyTerminationPolicyType**](EarlyTerminationPolicyType.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


