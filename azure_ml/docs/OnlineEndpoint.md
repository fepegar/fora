# OnlineEndpoint

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**compute** | Option<**String**> | ARM resource ID of the compute if it exists.  optional | [optional]
**mirror_traffic** | Option<**std::collections::HashMap<String, i32>**> | Percentage of traffic to be mirrored to each deployment without using returned scoring. Traffic values need to sum to utmost 50. | [optional]
**provisioning_state** | Option<[**models::EndpointProvisioningState**](EndpointProvisioningState.md)> |  | [optional]
**public_network_access** | Option<[**models::PublicNetworkAccessType**](PublicNetworkAccessType.md)> |  | [optional]
**traffic** | Option<**std::collections::HashMap<String, i32>**> | Percentage of traffic from endpoint to divert to each deployment. Traffic values need to sum to 100. | [optional]
**auth_mode** | [**models::EndpointAuthMode**](EndpointAuthMode.md) |  | 
**description** | Option<**String**> | Description of the inference endpoint. | [optional]
**keys** | Option<[**models::EndpointAuthKeys**](EndpointAuthKeys.md)> |  | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | Property dictionary. Properties can be added, but not removed or altered. | [optional]
**scoring_uri** | Option<**String**> | Endpoint URI. | [optional][readonly]
**swagger_uri** | Option<**String**> | Endpoint Swagger URI. | [optional][readonly]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


