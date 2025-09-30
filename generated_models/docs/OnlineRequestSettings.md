# OnlineRequestSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**max_concurrent_requests_per_instance** | Option<**i32**> | The number of maximum concurrent requests per node allowed per deployment. Defaults to 1. | [optional][default to 1]
**max_queue_wait** | Option<**String**> | (Deprecated for Managed Online Endpoints) The maximum amount of time a request will stay in the queue in ISO 8601 format.  Defaults to 500ms.  (Now increase `request_timeout_ms` to account for any networking/queue delays) | [optional][default to PT0.5S]
**request_timeout** | Option<**String**> | The scoring timeout in ISO 8601 format.  Defaults to 5000ms. | [optional][default to PT5S]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


