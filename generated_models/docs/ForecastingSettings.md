# ForecastingSettings

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**country_or_region_for_holidays** | Option<**String**> | Country or region for holidays for forecasting tasks.  These should be ISO 3166 two-letter country/region codes, for example 'US' or 'GB'. | [optional]
**cv_step_size** | Option<**i32**> | Number of periods between the origin time of one CV fold and the next fold. For  example, if `CVStepSize` = 3 for daily data, the origin time for each fold will be  three days apart. | [optional]
**feature_lags** | Option<[**models::FeatureLags**](FeatureLags.md)> |  | [optional]
**forecast_horizon** | Option<[**models::ForecastHorizon**](ForecastHorizon.md)> |  | [optional]
**frequency** | Option<**String**> | When forecasting, this parameter represents the period with which the forecast is desired, for example daily, weekly, yearly, etc. The forecast frequency is dataset frequency by default. | [optional]
**seasonality** | Option<[**models::Seasonality**](Seasonality.md)> |  | [optional]
**short_series_handling_config** | Option<[**models::ShortSeriesHandlingConfiguration**](ShortSeriesHandlingConfiguration.md)> |  | [optional]
**target_aggregate_function** | Option<[**models::TargetAggregationFunction**](TargetAggregationFunction.md)> |  | [optional]
**target_lags** | Option<[**models::TargetLags**](TargetLags.md)> |  | [optional]
**target_rolling_window_size** | Option<[**models::TargetRollingWindowSize**](TargetRollingWindowSize.md)> |  | [optional]
**time_column_name** | Option<**String**> | The name of the time column. This parameter is required when forecasting to specify the datetime column in the input data used for building the time series and inferring its frequency. | [optional]
**time_series_id_column_names** | Option<**Vec<String>**> | The names of columns used to group a timeseries. It can be used to create multiple series.  If grain is not defined, the data set is assumed to be one time-series. This parameter is used with task type forecasting. | [optional]
**use_stl** | Option<[**models::UseStl**](UseStl.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


