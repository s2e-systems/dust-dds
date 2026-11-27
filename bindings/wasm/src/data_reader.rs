use dust_dds::infrastructure::qos::QosKind;
use wasm_bindgen::prelude::*;

use crate::{
    domain_participant::DomainParticipant,
    dynamic_type::DynamicType,
    infrastructure::{
        instance::InstanceHandle, qos::DataReaderQos, status::SubscriptionMatchedStatus,
    },
    result::{Result, into_js_error},
    sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE, SampleInfo},
    subscriber::Subscriber,
    topic_description::TopicDescription,
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

    fn convert_samples(
        &self,
        samples: Vec<dust_dds::infrastructure::sample_info::Sample<JsDdsData>>,
    ) -> Result<js_sys::Array> {
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

        self.convert_samples(samples)
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

        self.convert_samples(samples)
    }

    #[wasm_bindgen(js_name = readNextSample)]
    pub async fn read_next_sample(&self) -> Result<Sample> {
        let sample = self
            ._inner
            .read_next_sample()
            .await
            .map_err(into_js_error)?;
        let data = if let Some(dds_data) = sample.data {
            Some(convert_dynamic_data_to_js(
                &dds_data.0,
                &self.dynamic_type.inner,
            )?)
        } else {
            None
        };
        Ok(Sample {
            data,
            sample_info: SampleInfo::from(sample.sample_info),
        })
    }

    #[wasm_bindgen(js_name = takeNextSample)]
    pub async fn take_next_sample(&self) -> Result<Sample> {
        let sample = self
            ._inner
            .take_next_sample()
            .await
            .map_err(into_js_error)?;
        let data = if let Some(dds_data) = sample.data {
            Some(convert_dynamic_data_to_js(
                &dds_data.0,
                &self.dynamic_type.inner,
            )?)
        } else {
            None
        };
        Ok(Sample {
            data,
            sample_info: SampleInfo::from(sample.sample_info),
        })
    }

    #[wasm_bindgen(js_name = readInstance)]
    pub async fn read_instance(
        &self,
        a_handle: &InstanceHandle,
        max_samples: Option<i32>,
    ) -> Result<js_sys::Array> {
        let max_samples = max_samples.unwrap_or(i32::MAX);
        let samples = match self
            ._inner
            .read_instance(
                max_samples,
                a_handle.into(),
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

        self.convert_samples(samples)
    }

    #[wasm_bindgen(js_name = takeInstance)]
    pub async fn take_instance(
        &self,
        a_handle: &InstanceHandle,
        max_samples: Option<i32>,
    ) -> Result<js_sys::Array> {
        let max_samples = max_samples.unwrap_or(i32::MAX);
        let samples = match self
            ._inner
            .take_instance(
                max_samples,
                a_handle.into(),
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

        self.convert_samples(samples)
    }

    #[wasm_bindgen(js_name = readNextInstance)]
    pub async fn read_next_instance(
        &self,
        previous_handle: Option<InstanceHandle>,
        max_samples: Option<i32>,
    ) -> Result<js_sys::Array> {
        let max_samples = max_samples.unwrap_or(i32::MAX);
        let samples = match self
            ._inner
            .read_next_instance(
                max_samples,
                previous_handle.map(Into::into),
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

        self.convert_samples(samples)
    }

    #[wasm_bindgen(js_name = takeNextInstance)]
    pub async fn take_next_instance(
        &self,
        previous_handle: Option<InstanceHandle>,
        max_samples: Option<i32>,
    ) -> Result<js_sys::Array> {
        let max_samples = max_samples.unwrap_or(i32::MAX);
        let samples = match self
            ._inner
            .take_next_instance(
                max_samples,
                previous_handle.map(Into::into),
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

        self.convert_samples(samples)
    }

    #[wasm_bindgen(js_name = getSubscriber)]
    pub fn get_subscriber(&self) -> Subscriber {
        Subscriber::new(self._inner.get_subscriber())
    }

    #[wasm_bindgen(js_name = getTopicDescription)]
    pub fn get_topicdescription(&self) -> TopicDescription {
        use dust_dds::dds_async::topic_description::TopicDescriptionAsync;
        let td = self._inner.get_topicdescription();
        TopicDescription::new(
            DomainParticipant::new(td.get_participant()),
            td.get_type_name(),
            td.get_name(),
        )
    }

    #[wasm_bindgen(js_name = getQos)]
    pub async fn get_qos(&self) -> Result<DataReaderQos> {
        let qos = self._inner.get_qos().await.map_err(into_js_error)?;
        Ok(qos.into())
    }

    #[wasm_bindgen(js_name = setQos)]
    pub async fn set_qos(&self, qos: Option<DataReaderQos>) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self._inner.set_qos(qos).await.map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = waitForHistoricalData)]
    pub async fn wait_for_historical_data(&self) -> Result<()> {
        self._inner
            .wait_for_historical_data()
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getSubscriptionMatchedStatus)]
    pub async fn get_subscription_matched_status(&self) -> Result<SubscriptionMatchedStatus> {
        let status = self
            ._inner
            .get_subscription_matched_status()
            .await
            .map_err(into_js_error)?;
        Ok(status.into())
    }

    #[wasm_bindgen(js_name = getMatchedPublications)]
    pub async fn get_matched_publications(&self) -> Result<js_sys::Array> {
        let handles = self
            ._inner
            .get_matched_publications()
            .await
            .map_err(into_js_error)?;
        let array = js_sys::Array::new();
        for h in handles {
            array.push(&JsValue::from(InstanceHandle::from(h)));
        }
        Ok(array)
    }

    pub async fn enable(&self) -> Result<()> {
        self._inner.enable().await.map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getStatusCondition)]
    pub fn get_status_condition(&self) -> crate::condition::StatusCondition {
        crate::condition::StatusCondition::new(
            self._inner.get_statuscondition(),
            self._inner.get_instance_handle(),
        )
    }

    #[wasm_bindgen(js_name = setListener)]
    pub async fn set_listener(
        &self,
        a_listener: Option<js_sys::Object>,
        mask: Option<js_sys::Array>,
    ) -> Result<()> {
        let listener = a_listener.map(|l| crate::listener::WasmDataReaderListener {
            listener: crate::listener::SendJsValue(l.into()),
            dynamic_type: self.dynamic_type.clone(),
        });
        let status_mask = crate::listener::parse_status_mask(mask)?;
        self._inner
            .set_listener(listener, &status_mask)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }
}
