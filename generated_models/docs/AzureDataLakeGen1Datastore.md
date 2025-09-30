# AzureDataLakeGen1Datastore

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**service_data_access_auth_identity** | Option<[**models::ServiceDataAccessAuthIdentity**](ServiceDataAccessAuthIdentity.md)> |  | [optional]
**store_name** | **String** | [Required] Azure Data Lake store name. | 
**resource_group** | Option<**String**> | Azure Resource Group name | [optional]
**subscription_id** | Option<**String**> | Azure Subscription Id | [optional]
**credentials** | [**models::DatastoreCredentials**](DatastoreCredentials.md) |  | 
**datastore_type** | [**models::DatastoreType**](DatastoreType.md) |  | 
**is_default** | Option<**bool**> | Readonly property to indicate if datastore is the workspace default datastore | [optional][readonly]
**description** | Option<**String**> | The asset description text. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | The asset property dictionary. | [optional]
**tags** | Option<**std::collections::HashMap<String, String>**> | Tag dictionary. Tags can be added, removed, and updated. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


