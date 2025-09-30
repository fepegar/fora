# AzureFileDatastore

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_name** | **String** | [Required] Storage account name. | 
**endpoint** | Option<**String**> | Azure cloud endpoint for the storage account. | [optional]
**file_share_name** | **String** | [Required] The name of the Azure file share that the datastore points to. | 
**protocol** | Option<**String**> | Protocol used to communicate with the storage account. | [optional]
**service_data_access_auth_identity** | Option<[**models::ServiceDataAccessAuthIdentity**](ServiceDataAccessAuthIdentity.md)> |  | [optional]
**resource_group** | Option<**String**> | Azure Resource Group name | [optional]
**subscription_id** | Option<**String**> | Azure Subscription Id | [optional]
**credentials** | [**models::DatastoreCredentials**](DatastoreCredentials.md) |  | 
**datastore_type** | [**models::DatastoreType**](DatastoreType.md) |  | 
**is_default** | Option<**bool**> | Readonly property to indicate if datastore is the workspace default datastore | [optional][readonly]
**description** | Option<**String**> | The asset description text. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | The asset property dictionary. | [optional]
**tags** | Option<**std::collections::HashMap<String, String>**> | Tag dictionary. Tags can be added, removed, and updated. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


