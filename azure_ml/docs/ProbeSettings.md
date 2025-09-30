# ProbeSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**failure_threshold** | Option<**i32**> | The number of failures to allow before returning an unhealthy status. | [optional][default to 30]
**initial_delay** | Option<**String**> | The delay before the first probe in ISO 8601 format. | [optional]
**period** | Option<**String**> | The length of time between probes in ISO 8601 format. | [optional][default to PT10S]
**success_threshold** | Option<**i32**> | The number of successful probes before returning a healthy status. | [optional][default to 1]
**timeout** | Option<**String**> | The probe timeout in ISO 8601 format. | [optional][default to PT2S]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


