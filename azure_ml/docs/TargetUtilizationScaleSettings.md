# TargetUtilizationScaleSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**max_instances** | Option<**i32**> | The maximum number of instances that the deployment can scale to. The quota will be reserved for max_instances. | [optional][default to 1]
**min_instances** | Option<**i32**> | The minimum number of instances to always be present. | [optional][default to 1]
**polling_interval** | Option<**String**> | The polling interval in ISO 8691 format. Only supports duration with precision as low as Seconds. | [optional][default to PT1S]
**target_utilization_percentage** | Option<**i32**> | Target CPU usage for the autoscaler. | [optional][default to 70]
**scale_type** | [**models::ScaleType**](ScaleType.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


