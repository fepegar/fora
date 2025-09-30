# BatchEndpointTrackedResource

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**identity** | Option<[**models::ManagedServiceIdentity**](ManagedServiceIdentity.md)> |  | [optional]
**kind** | Option<**String**> | Metadata used by portal/tooling/etc to render different UX experiences for resources of the same type. | [optional]
**properties** | [**models::BatchEndpoint**](BatchEndpoint.md) |  | 
**sku** | Option<[**models::Sku**](Sku.md)> |  | [optional]
**tags** | Option<**std::collections::HashMap<String, String>**> | Resource tags. | [optional]
**location** | **String** | The geo-location where the resource lives | 
**id** | Option<**String**> | Fully qualified resource ID for the resource. Ex - /subscriptions/{subscriptionId}/resourceGroups/{resourceGroupName}/providers/{resourceProviderNamespace}/{resourceType}/{resourceName} | [optional][readonly]
**name** | Option<**String**> | The name of the resource | [optional][readonly]
**r#type** | Option<**String**> | The type of the resource. E.g. \"Microsoft.Compute/virtualMachines\" or \"Microsoft.Storage/storageAccounts\" | [optional][readonly]
**system_data** | Option<[**models::SystemData**](systemData.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


