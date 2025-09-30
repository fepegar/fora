# Schedule

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**action** | [**models::ScheduleActionBase**](ScheduleActionBase.md) |  | 
**display_name** | Option<**String**> | Display name of schedule. | [optional]
**is_enabled** | Option<**bool**> | Is the schedule enabled? | [optional][default to true]
**provisioning_state** | Option<[**models::ScheduleProvisioningStatus**](ScheduleProvisioningStatus.md)> |  | [optional]
**trigger** | [**models::TriggerBase**](TriggerBase.md) |  | 
**description** | Option<**String**> | The asset description text. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | The asset property dictionary. | [optional]
**tags** | Option<**std::collections::HashMap<String, String>**> | Tag dictionary. Tags can be added, removed, and updated. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


