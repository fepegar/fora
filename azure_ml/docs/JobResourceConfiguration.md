# JobResourceConfiguration

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**docker_args** | Option<**String**> | Extra arguments to pass to the Docker run command. This would override any parameters that have already been set by the system, or in this section. This parameter is only supported for Azure ML compute types. | [optional]
**shm_size** | Option<**String**> | Size of the docker container's shared memory block. This should be in the format of (number)(unit) where number as to be greater than 0 and the unit can be one of b(bytes), k(kilobytes), m(megabytes), or g(gigabytes). | [optional][default to 2g]
**instance_count** | Option<**i32**> | Optional number of instances or nodes used by the compute target. | [optional][default to 1]
**instance_type** | Option<**String**> | Optional type of VM used as supported by the compute target. | [optional]
**properties** | Option<[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)> | Additional properties bag. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


