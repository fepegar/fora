# CodeVersion

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**code_uri** | Option<**String**> | Uri where code is located | [optional]
**provisioning_state** | Option<[**models::AssetProvisioningState**](AssetProvisioningState.md)> |  | [optional]
**is_anonymous** | Option<**bool**> | If the name version are system generated (anonymous registration). | [optional][default to false]
**is_archived** | Option<**bool**> | Is the asset archived? | [optional][default to false]
**description** | Option<**String**> | The asset description text. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | The asset property dictionary. | [optional]
**tags** | Option<**std::collections::HashMap<String, String>**> | Tag dictionary. Tags can be added, removed, and updated. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


