# BuildContext

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**context_uri** | **String** | [Required] URI of the Docker build context used to build the image. Supports blob URIs on environment creation and may return blob or Git URIs.  <seealso href=\"https://docs.docker.com/engine/reference/commandline/build/#extended-description\" /> | 
**dockerfile_path** | Option<**String**> | Path to the Dockerfile in the build context.  <seealso href=\"https://docs.docker.com/engine/reference/builder/\" /> | [optional][default to Dockerfile]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


