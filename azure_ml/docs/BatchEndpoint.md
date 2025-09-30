# BatchEndpoint

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**defaults** | Option<[**models::BatchEndpointDefaults**](BatchEndpointDefaults.md)> |  | [optional]
**provisioning_state** | Option<[**models::EndpointProvisioningState**](EndpointProvisioningState.md)> |  | [optional]
**auth_mode** | [**models::EndpointAuthMode**](EndpointAuthMode.md) |  | 
**description** | Option<**String**> | Description of the inference endpoint. | [optional]
**keys** | Option<[**models::EndpointAuthKeys**](EndpointAuthKeys.md)> |  | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | Property dictionary. Properties can be added, but not removed or altered. | [optional]
**scoring_uri** | Option<**String**> | Endpoint URI. | [optional][readonly]
**swagger_uri** | Option<**String**> | Endpoint Swagger URI. | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


