# ModelVersion

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**flavors** | Option<[**std::collections::HashMap<String, models::FlavorData>**](FlavorData.md)> | Mapping of model flavors to their properties. | [optional]
**job_name** | Option<**String**> | Name of the training job which produced this model | [optional]
**model_type** | Option<**String**> | The storage format for this entity. Used for NCD. | [optional]
**model_uri** | Option<**String**> | The URI path to the model contents. | [optional]
**provisioning_state** | Option<[**models::AssetProvisioningState**](AssetProvisioningState.md)> |  | [optional]
**stage** | Option<**String**> | Stage in the model lifecycle assigned to this model | [optional]
**is_anonymous** | Option<**bool**> | If the name version are system generated (anonymous registration). | [optional][default to false]
**is_archived** | Option<**bool**> | Is the asset archived? | [optional][default to false]
**description** | Option<**String**> | The asset description text. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | The asset property dictionary. | [optional]
**tags** | Option<**std::collections::HashMap<String, String>**> | Tag dictionary. Tags can be added, removed, and updated. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


