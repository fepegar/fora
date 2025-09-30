# Sku

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The name of the SKU. Ex - P3. It is typically a letter+number code | 
**tier** | Option<[**models::SkuTier**](SkuTier.md)> |  | [optional]
**size** | Option<**String**> | The SKU size. When the name field is the combination of tier and some other value, this would be the standalone code.  | [optional]
**family** | Option<**String**> | If the service has different generations of hardware, for the same SKU, then that can be captured here. | [optional]
**capacity** | Option<**i32**> | If the SKU supports scale out/in then the capacity integer should be included. If scale out/in is not possible for the resource this may be omitted. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


