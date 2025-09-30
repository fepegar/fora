# ServicePrincipalDatastoreCredentials

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**authority_url** | Option<**String**> | Authority URL used for authentication. | [optional]
**client_id** | [**uuid::Uuid**](uuid::Uuid.md) | [Required] Service principal client ID. | 
**resource_url** | Option<**String**> | Resource the service principal has access to. | [optional]
**secrets** | [**models::ServicePrincipalDatastoreSecrets**](ServicePrincipalDatastoreSecrets.md) |  | 
**tenant_id** | [**uuid::Uuid**](uuid::Uuid.md) | [Required] ID of the tenant to which the service principal belongs. | 
**credentials_type** | [**models::CredentialsType**](CredentialsType.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


