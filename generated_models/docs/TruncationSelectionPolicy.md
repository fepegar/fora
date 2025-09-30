# TruncationSelectionPolicy

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**truncation_percentage** | Option<**i32**> | The percentage of runs to cancel at each evaluation interval. | [optional][default to 0]
**delay_evaluation** | Option<**i32**> | Number of intervals by which to delay the first evaluation. | [optional][default to 0]
**evaluation_interval** | Option<**i32**> | Interval (number of runs) between policy evaluations. | [optional][default to 0]
**policy_type** | [**models::EarlyTerminationPolicyType**](EarlyTerminationPolicyType.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


