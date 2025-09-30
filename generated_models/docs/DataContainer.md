# DataContainer

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**data_type** | [**models::DataType**](DataType.md) |  | 
**is_archived** | Option<**bool**> | Is the asset archived? | [optional][default to false]
**latest_version** | Option<**String**> | The latest version inside this container. | [optional][readonly]
**next_version** | Option<**String**> | The next auto incremental version | [optional][readonly]
**description** | Option<**String**> | The asset description text. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | The asset property dictionary. | [optional]
**tags** | Option<**std::collections::HashMap<String, String>**> | Tag dictionary. Tags can be added, removed, and updated. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


