# FeaturesetVersionBackfillRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**data_availability_status** | Option<[**Vec<models::DataAvailabilityStatus>**](DataAvailabilityStatus.md)> | Specified the data availability status that you want to backfill | [optional]
**description** | Option<**String**> | Specifies description | [optional]
**display_name** | Option<**String**> | Specifies description | [optional]
**feature_window** | Option<[**models::FeatureWindow**](FeatureWindow.md)> |  | [optional]
**job_id** | Option<**String**> | Specify the jobId to retry the failed materialization | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | Specifies the properties | [optional]
**resource** | Option<[**models::MaterializationComputeResource**](MaterializationComputeResource.md)> |  | [optional]
**spark_configuration** | Option<**std::collections::HashMap<String, String>**> | Specifies the spark compute settings | [optional]
**tags** | Option<**std::collections::HashMap<String, String>**> | Specifies the tags | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


