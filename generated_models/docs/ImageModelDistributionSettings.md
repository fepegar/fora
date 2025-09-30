# ImageModelDistributionSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ams_gradient** | Option<**String**> | Enable AMSGrad when optimizer is 'adam' or 'adamw'. | [optional]
**augmentations** | Option<**String**> | Settings for using Augmentations. | [optional]
**beta1** | Option<**String**> | Value of 'beta1' when optimizer is 'adam' or 'adamw'. Must be a float in the range [0, 1]. | [optional]
**beta2** | Option<**String**> | Value of 'beta2' when optimizer is 'adam' or 'adamw'. Must be a float in the range [0, 1]. | [optional]
**distributed** | Option<**String**> | Whether to use distributer training. | [optional]
**early_stopping** | Option<**String**> | Enable early stopping logic during training. | [optional]
**early_stopping_delay** | Option<**String**> | Minimum number of epochs or validation evaluations to wait before primary metric improvement  is tracked for early stopping. Must be a positive integer. | [optional]
**early_stopping_patience** | Option<**String**> | Minimum number of epochs or validation evaluations with no primary metric improvement before  the run is stopped. Must be a positive integer. | [optional]
**enable_onnx_normalization** | Option<**String**> | Enable normalization when exporting ONNX model. | [optional]
**evaluation_frequency** | Option<**String**> | Frequency to evaluate validation dataset to get metric scores. Must be a positive integer. | [optional]
**gradient_accumulation_step** | Option<**String**> | Gradient accumulation means running a configured number of \"GradAccumulationStep\" steps without  updating the model weights while accumulating the gradients of those steps, and then using  the accumulated gradients to compute the weight updates. Must be a positive integer. | [optional]
**layers_to_freeze** | Option<**String**> | Number of layers to freeze for the model. Must be a positive integer.  For instance, passing 2 as value for 'seresnext' means  freezing layer0 and layer1. For a full list of models supported and details on layer freeze, please  see: https://docs.microsoft.com/en-us/azure/machine-learning/how-to-auto-train-image-models. | [optional]
**learning_rate** | Option<**String**> | Initial learning rate. Must be a float in the range [0, 1]. | [optional]
**learning_rate_scheduler** | Option<**String**> | Type of learning rate scheduler. Must be 'warmup_cosine' or 'step'. | [optional]
**model_name** | Option<**String**> | Name of the model to use for training.  For more information on the available models please visit the official documentation:  https://docs.microsoft.com/en-us/azure/machine-learning/how-to-auto-train-image-models. | [optional]
**momentum** | Option<**String**> | Value of momentum when optimizer is 'sgd'. Must be a float in the range [0, 1]. | [optional]
**nesterov** | Option<**String**> | Enable nesterov when optimizer is 'sgd'. | [optional]
**number_of_epochs** | Option<**String**> | Number of training epochs. Must be a positive integer. | [optional]
**number_of_workers** | Option<**String**> | Number of data loader workers. Must be a non-negative integer. | [optional]
**optimizer** | Option<**String**> | Type of optimizer. Must be either 'sgd', 'adam', or 'adamw'. | [optional]
**random_seed** | Option<**String**> | Random seed to be used when using deterministic training. | [optional]
**step_lr_gamma** | Option<**String**> | Value of gamma when learning rate scheduler is 'step'. Must be a float in the range [0, 1]. | [optional]
**step_lr_step_size** | Option<**String**> | Value of step size when learning rate scheduler is 'step'. Must be a positive integer. | [optional]
**training_batch_size** | Option<**String**> | Training batch size. Must be a positive integer. | [optional]
**validation_batch_size** | Option<**String**> | Validation batch size. Must be a positive integer. | [optional]
**warmup_cosine_lr_cycles** | Option<**String**> | Value of cosine cycle when learning rate scheduler is 'warmup_cosine'. Must be a float in the range [0, 1]. | [optional]
**warmup_cosine_lr_warmup_epochs** | Option<**String**> | Value of warmup epochs when learning rate scheduler is 'warmup_cosine'. Must be a positive integer. | [optional]
**weight_decay** | Option<**String**> | Value of weight decay when optimizer is 'sgd', 'adam', or 'adamw'. Must be a float in the range[0, 1]. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


