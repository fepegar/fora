# KubernetesOnlineDeployment

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**container_resource_requirements** | Option<[**models::ContainerResourceRequirements**](ContainerResourceRequirements.md)> |  | [optional]
**app_insights_enabled** | Option<**bool**> | If true, enables Application Insights logging. | [optional][default to false]
**data_collector** | Option<[**models::DataCollector**](DataCollector.md)> |  | [optional]
**egress_public_network_access** | Option<[**models::EgressPublicNetworkAccessType**](EgressPublicNetworkAccessType.md)> |  | [optional]
**endpoint_compute_type** | [**models::EndpointComputeType**](EndpointComputeType.md) |  | 
**instance_type** | Option<**String**> | Compute instance type. Default: Standard_F4s_v2. | [optional][default to Standard_F4s_v2]
**liveness_probe** | Option<[**models::ProbeSettings**](ProbeSettings.md)> |  | [optional]
**model** | Option<**String**> | The URI path to the model. | [optional]
**model_mount_path** | Option<**String**> | The path to mount the model in custom container. | [optional]
**provisioning_state** | Option<[**models::DeploymentProvisioningState**](DeploymentProvisioningState.md)> |  | [optional]
**readiness_probe** | Option<[**models::ProbeSettings**](ProbeSettings.md)> |  | [optional]
**startup_probe** | Option<[**models::ProbeSettings**](ProbeSettings.md)> |  | [optional]
**request_settings** | Option<[**models::OnlineRequestSettings**](OnlineRequestSettings.md)> |  | [optional]
**scale_settings** | Option<[**models::OnlineScaleSettings**](OnlineScaleSettings.md)> |  | [optional]
**code_configuration** | Option<[**models::CodeConfiguration**](CodeConfiguration.md)> |  | [optional]
**description** | Option<**String**> | Description of the endpoint deployment. | [optional]
**environment_id** | Option<**String**> | ARM resource ID or AssetId of the environment specification for the endpoint deployment. | [optional]
**environment_variables** | Option<**std::collections::HashMap<String, String>**> | Environment variables configuration for the deployment. | [optional]
**properties** | Option<**std::collections::HashMap<String, String>**> | Property dictionary. Properties can be added, but not removed or altered. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


