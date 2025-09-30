# ManagedIdentity

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**client_id** | Option<[**uuid::Uuid**](uuid::Uuid.md)> | Specifies a user-assigned identity by client ID. For system-assigned, do not set this field. | [optional]
**object_id** | Option<[**uuid::Uuid**](uuid::Uuid.md)> | Specifies a user-assigned identity by object ID. For system-assigned, do not set this field. | [optional]
**resource_id** | Option<**String**> | Specifies a user-assigned identity by ARM resource ID. For system-assigned, do not set this field. | [optional]
**identity_type** | [**models::IdentityConfigurationType**](IdentityConfigurationType.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


