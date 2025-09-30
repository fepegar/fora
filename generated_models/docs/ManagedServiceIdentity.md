# ManagedServiceIdentity

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**principal_id** | Option<[**uuid::Uuid**](uuid::Uuid.md)> | The service principal ID of the system assigned identity. This property will only be provided for a system assigned identity. | [optional][readonly]
**tenant_id** | Option<[**uuid::Uuid**](uuid::Uuid.md)> | The tenant ID of the system assigned identity. This property will only be provided for a system assigned identity. | [optional][readonly]
**r#type** | [**models::ManagedServiceIdentityType**](ManagedServiceIdentityType.md) |  | 
**user_assigned_identities** | Option<[**std::collections::HashMap<String, models::UserAssignedIdentity>**](UserAssignedIdentity.md)> | The set of user assigned identities associated with the resource. The userAssignedIdentities dictionary keys will be ARM resource ids in the form: '/subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/Microsoft.ManagedIdentity/userAssignedIdentities/{identityName}. The dictionary values can be empty objects ({}) in requests. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


