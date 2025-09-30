# CronTrigger

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**expression** | **String** | [Required] Specifies cron expression of schedule.  The expression should follow NCronTab format. | 
**end_time** | Option<**String**> | Specifies end time of schedule in ISO 8601, but without a UTC offset. Refer https://en.wikipedia.org/wiki/ISO_8601.  Recommented format would be \"2022-06-01T00:00:01\"  If not present, the schedule will run indefinitely | [optional]
**start_time** | Option<**String**> | Specifies start time of schedule in ISO 8601 format, but without a UTC offset. | [optional]
**time_zone** | Option<**String**> | Specifies time zone in which the schedule runs.  TimeZone should follow Windows time zone format. Refer: https://docs.microsoft.com/en-us/windows-hardware/manufacture/desktop/default-time-zones?view=windows-11 | [optional][default to UTC]
**trigger_type** | [**models::TriggerType**](TriggerType.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


