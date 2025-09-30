# Collection

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**client_id** | Option<**String**> | The msi client id used to collect logging to blob storage. If it's null,backend will pick a registered endpoint identity to auth. | [optional]
**data_collection_mode** | Option<[**models::DataCollectionMode**](DataCollectionMode.md)> |  | [optional]
**data_id** | Option<**String**> | The data asset arm resource id. Client side will ensure data asset is pointing to the blob storage, and backend will collect data to the blob storage. | [optional]
**sampling_rate** | Option<**f64**> | The sampling rate for collection. Sampling rate 1.0 means we collect 100% of data by default. | [optional][default to 1.0]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


