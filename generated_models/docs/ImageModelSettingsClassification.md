# ImageModelSettingsClassification

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**training_crop_size** | Option<**i32**> | Image crop size that is input to the neural network for the training dataset. Must be a positive integer. | [optional]
**validation_crop_size** | Option<**i32**> | Image crop size that is input to the neural network for the validation dataset. Must be a positive integer. | [optional]
**validation_resize_size** | Option<**i32**> | Image size to which to resize before cropping for validation dataset. Must be a positive integer. | [optional]
**weighted_loss** | Option<**i32**> | Weighted loss. The accepted values are 0 for no weighted loss.  1 for weighted loss with sqrt.(class_weights). 2 for weighted loss with class_weights. Must be 0 or 1 or 2. | [optional]
**advanced_settings** | Option<**String**> | Settings for advanced scenarios. | [optional]
**ams_gradient** | Option<**bool**> | Enable AMSGrad when optimizer is 'adam' or 'adamw'. | [optional]
**augmentations** | Option<**String**> | Settings for using Augmentations. | [optional]
**beta1** | Option<**f32**> | Value of 'beta1' when optimizer is 'adam' or 'adamw'. Must be a float in the range [0, 1]. | [optional]
**beta2** | Option<**f32**> | Value of 'beta2' when optimizer is 'adam' or 'adamw'. Must be a float in the range [0, 1]. | [optional]
**checkpoint_frequency** | Option<**i32**> | Frequency to store model checkpoints. Must be a positive integer. | [optional]
**checkpoint_model** | Option<[**models::MlFlowModelJobInput**](MLFlowModelJobInput.md)> |  | [optional]
**checkpoint_run_id** | Option<**String**> | The id of a previous run that has a pretrained checkpoint for incremental training. | [optional]
**distributed** | Option<**bool**> | Whether to use distributed training. | [optional]
**early_stopping** | Option<**bool**> | Enable early stopping logic during training. | [optional]
**early_stopping_delay** | Option<**i32**> | Minimum number of epochs or validation evaluations to wait before primary metric improvement  is tracked for early stopping. Must be a positive integer. | [optional]
**early_stopping_patience** | Option<**i32**> | Minimum number of epochs or validation evaluations with no primary metric improvement before  the run is stopped. Must be a positive integer. | [optional]
**enable_onnx_normalization** | Option<**bool**> | Enable normalization when exporting ONNX model. | [optional]
**evaluation_frequency** | Option<**i32**> | Frequency to evaluate validation dataset to get metric scores. Must be a positive integer. | [optional]
**gradient_accumulation_step** | Option<**i32**> | Gradient accumulation means running a configured number of \"GradAccumulationStep\" steps without  updating the model weights while accumulating the gradients of those steps, and then using  the accumulated gradients to compute the weight updates. Must be a positive integer. | [optional]
**layers_to_freeze** | Option<**i32**> | Number of layers to freeze for the model. Must be a positive integer.  For instance, passing 2 as value for 'seresnext' means  freezing layer0 and layer1. For a full list of models supported and details on layer freeze, please  see: https://docs.microsoft.com/en-us/azure/machine-learning/how-to-auto-train-image-models. | [optional]
**learning_rate** | Option<**f32**> | Initial learning rate. Must be a float in the range [0, 1]. | [optional]
**learning_rate_scheduler** | Option<[**models::LearningRateScheduler**](LearningRateScheduler.md)> |  | [optional]
**model_name** | Option<**String**> | Name of the model to use for training.  For more information on the available models please visit the official documentation:  https://docs.microsoft.com/en-us/azure/machine-learning/how-to-auto-train-image-models. | [optional]
**momentum** | Option<**f32**> | Value of momentum when optimizer is 'sgd'. Must be a float in the range [0, 1]. | [optional]
**nesterov** | Option<**bool**> | Enable nesterov when optimizer is 'sgd'. | [optional]
**number_of_epochs** | Option<**i32**> | Number of training epochs. Must be a positive integer. | [optional]
**number_of_workers** | Option<**i32**> | Number of data loader workers. Must be a non-negative integer. | [optional]
**optimizer** | Option<[**models::StochasticOptimizer**](StochasticOptimizer.md)> |  | [optional]
**random_seed** | Option<**i32**> | Random seed to be used when using deterministic training. | [optional]
**step_lr_gamma** | Option<**f32**> | Value of gamma when learning rate scheduler is 'step'. Must be a float in the range [0, 1]. | [optional]
**step_lr_step_size** | Option<**i32**> | Value of step size when learning rate scheduler is 'step'. Must be a positive integer. | [optional]
**training_batch_size** | Option<**i32**> | Training batch size. Must be a positive integer. | [optional]
**validation_batch_size** | Option<**i32**> | Validation batch size. Must be a positive integer. | [optional]
**warmup_cosine_lr_cycles** | Option<**f32**> | Value of cosine cycle when learning rate scheduler is 'warmup_cosine'. Must be a float in the range [0, 1]. | [optional]
**warmup_cosine_lr_warmup_epochs** | Option<**i32**> | Value of warmup epochs when learning rate scheduler is 'warmup_cosine'. Must be a positive integer. | [optional]
**weight_decay** | Option<**f32**> | Value of weight decay when optimizer is 'sgd', 'adam', or 'adamw'. Must be a float in the range[0, 1]. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


