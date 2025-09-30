# NotificationSetting

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**email_on** | Option<[**Vec<models::EmailNotificationEnableType>**](EmailNotificationEnableType.md)> | Send email notification to user on specified notification type | [optional]
**emails** | Option<**Vec<String>**> | This is the email recipient list which has a limitation of 499 characters in total concat with comma separator | [optional]
**webhooks** | Option<[**std::collections::HashMap<String, models::Webhook>**](Webhook.md)> | Send webhook callback to a service. Key is a user-provided name for the webhook. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


