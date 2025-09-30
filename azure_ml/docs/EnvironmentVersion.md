# EnvironmentVersion

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**auto_rebuild** | Option<[**models::AutoRebuildSetting**](AutoRebuildSetting.md)> |  | [optional]
**build** | Option<[**models::BuildContext**](BuildContext.md)> |  | [optional]
**conda_file** | Option<**String**> | Standard configuration file used by Conda that lets you install any kind of package, including Python, R, and C/C++ packages.  <see href=\"https://repo2docker.readthedocs.io/en/latest/config_files.html#environment-yml-install-a-conda-environment\" /> | [optional]
**environment_type** | Option<[**models::EnvironmentType**](EnvironmentType.md)> |  | [optional]
**image** | Option<**String**> | Name of the image that will be used for the environment.  <seealso href=\"https://docs.microsoft.com/en-us/azure/machine-learning/how-to-deploy-custom-docker-image#use-a-custom-base-image\" /> | [optional]
**inference_config** | Option<[**models::InferenceContainerProperties**](InferenceContainerProperties.md)> |  | [optional]
**os_type** | Option<[**models::OperatingSystemType**](OperatingSystemType.md)> |  | [optional]
**provisioning_state** | Option<[**models::AssetProvisioningState**](AssetProvisioningState.md)> |  | [optional]
**stage** | Option<**String**> | Stage in the environment lifecycle assigned to this environment | [optional]
**is_anonymous** | Option<**bool**> | If the name version are system generated (anonymous registration). | [optional][default to false]
**is_archived** | Option<**bool**> | Is the asset archived? | [optional][default to false]
**description** | Option<**String**> | The asset description text. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | The asset property dictionary. | [optional]
**tags** | Option<**std::collections::HashMap<String, String>**> | Tag dictionary. Tags can be added, removed, and updated. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


