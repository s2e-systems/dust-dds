use wasm_bindgen::prelude::*;

use crate::{
    dynamic_type::DynamicType,
    infrastructure::instance::InstanceHandle,
    result::{Result, into_js_error},
    sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE, SampleInfo},
    type_support::{JsDdsData, convert_dynamic_data_to_js},
};

#[wasm_bindgen]
pub struct Sample {
    data: Option<JsValue>,
    sample_info: SampleInfo,
}

#[wasm_bindgen]
impl Sample {
    #[wasm_bindgen(js_name = getData)]
    pub fn get_data(&self) -> Result<JsValue> {
        self.data
            .clone()
            .ok_or_else(|| js_sys::Error::new("No data available").into())
    }

    #[wasm_bindgen(js_name = getSampleInfo)]
    pub fn get_sample_info(&self) -> SampleInfo {
        self.sample_info.clone()
    }
}

#[wasm_bindgen]
pub struct DataReader {
    pub(crate) _inner: dust_dds::dds_async::data_reader::DataReaderAsync<JsDdsData>,
    pub(crate) dynamic_type: DynamicType,
}

impl DataReader {
    pub(crate) fn new(
        inner: dust_dds::dds_async::data_reader::DataReaderAsync<JsDdsData>,
        dynamic_type: DynamicType,
    ) -> Self {
        Self {
            _inner: inner,
            dynamic_type,
        }
    }
}

#[wasm_bindgen]
impl DataReader {
    #[wasm_bindgen(js_name = getInstanceHandle)]
    pub fn get_instance_handle(&self) -> InstanceHandle {
        self._inner.get_instance_handle().into()
    }

    #[wasm_bindgen(js_name = read)]
    pub async fn read(&self, max_samples: Option<i32>) -> Result<js_sys::Array> {
        let max_samples = max_samples.unwrap_or(i32::MAX);
        let samples = match self
            ._inner
            .read(
                max_samples,
                ANY_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
            )
            .await
        {
            Ok(s) => s,
            Err(dust_dds::infrastructure::error::DdsError::NoData) => {
                return Ok(js_sys::Array::new());
            }
            Err(e) => return Err(into_js_error(e)),
        };

        let result = js_sys::Array::new();
        for sample in samples {
            let data = if let Some(dds_data) = sample.data {
                Some(convert_dynamic_data_to_js(
                    &dds_data.0,
                    &self.dynamic_type.inner,
                )?)
            } else {
                None
            };
            let js_sample = Sample {
                data,
                sample_info: SampleInfo::from(sample.sample_info),
            };
            result.push(&JsValue::from(js_sample));
        }
        Ok(result)
    }

    #[wasm_bindgen(js_name = take)]
    pub async fn take(&self, max_samples: Option<i32>) -> Result<js_sys::Array> {
        let max_samples = max_samples.unwrap_or(i32::MAX);
        let samples = match self
            ._inner
            .take(
                max_samples,
                ANY_SAMPLE_STATE,
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
            )
            .await
        {
            Ok(s) => s,
            Err(dust_dds::infrastructure::error::DdsError::NoData) => {
                return Ok(js_sys::Array::new());
            }
            Err(e) => return Err(into_js_error(e)),
        };

        let result = js_sys::Array::new();
        for sample in samples {
            let data = if let Some(dds_data) = sample.data {
                Some(convert_dynamic_data_to_js(
                    &dds_data.0,
                    &self.dynamic_type.inner,
                )?)
            } else {
                None
            };
            let js_sample = Sample {
                data,
                sample_info: SampleInfo::from(sample.sample_info),
            };
            result.push(&JsValue::from(js_sample));
        }
        Ok(result)
    }

    #[wasm_bindgen(js_name = getSubscriptionMatchedStatus)]
    pub async fn get_subscription_matched_status(
        &self,
    ) -> Result<crate::infrastructure::status::SubscriptionMatchedStatus> {
        let status = self
            ._inner
            .get_subscription_matched_status()
            .await
            .map_err(into_js_error)?;
        Ok(status.into())
    }
}
