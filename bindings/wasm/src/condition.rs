use wasm_bindgen::prelude::*;

use crate::{Result, infrastructure::status::StatusKind, result::into_js_error};

#[wasm_bindgen]
#[derive(Clone)]
pub struct StatusCondition {
    pub(crate) inner: dust_dds::dds_async::condition::StatusConditionAsync,
    pub(crate) entity_handle: dust_dds::infrastructure::instance::InstanceHandle,
}

impl PartialEq for StatusCondition {
    fn eq(&self, other: &Self) -> bool {
        self.entity_handle == other.entity_handle
    }
}
impl Eq for StatusCondition {}

impl StatusCondition {
    pub(crate) fn new(
        inner: dust_dds::dds_async::condition::StatusConditionAsync,
        entity_handle: dust_dds::infrastructure::instance::InstanceHandle,
    ) -> Self {
        Self {
            inner,
            entity_handle,
        }
    }
}

#[wasm_bindgen]
impl StatusCondition {
    #[wasm_bindgen(js_name = getEnabledStatuses)]
    pub async fn get_enabled_statuses(&self) -> Result<js_sys::Array> {
        let statuses = self
            .inner
            .get_enabled_statuses()
            .await
            .map_err(into_js_error)?;
        let array = js_sys::Array::new();
        for status in statuses {
            array.push(&JsValue::from(StatusKind::from(status)));
        }
        Ok(array)
    }

    #[wasm_bindgen(js_name = setEnabledStatuses)]
    pub async fn set_enabled_statuses(&self, mask: &js_sys::Array) -> Result<()> {
        let mut rust_mask = Vec::new();
        for val in mask.iter() {
            let num = val.as_f64().ok_or_else(|| {
                into_js_error(dust_dds::infrastructure::error::DdsError::BadParameter)
            })? as u32;
            let status = match num {
                0 => dust_dds::infrastructure::status::StatusKind::InconsistentTopic,
                1 => dust_dds::infrastructure::status::StatusKind::OfferedDeadlineMissed,
                2 => dust_dds::infrastructure::status::StatusKind::RequestedDeadlineMissed,
                3 => dust_dds::infrastructure::status::StatusKind::OfferedIncompatibleQos,
                4 => dust_dds::infrastructure::status::StatusKind::RequestedIncompatibleQos,
                5 => dust_dds::infrastructure::status::StatusKind::SampleLost,
                6 => dust_dds::infrastructure::status::StatusKind::SampleRejected,
                7 => dust_dds::infrastructure::status::StatusKind::DataOnReaders,
                8 => dust_dds::infrastructure::status::StatusKind::DataAvailable,
                9 => dust_dds::infrastructure::status::StatusKind::LivelinessLost,
                10 => dust_dds::infrastructure::status::StatusKind::LivelinessChanged,
                11 => dust_dds::infrastructure::status::StatusKind::PublicationMatched,
                12 => dust_dds::infrastructure::status::StatusKind::SubscriptionMatched,
                _ => {
                    return Err(into_js_error(
                        dust_dds::infrastructure::error::DdsError::BadParameter,
                    ));
                }
            };
            rust_mask.push(status);
        }
        self.inner
            .set_enabled_statuses(&rust_mask)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getTriggerValue)]
    pub async fn get_trigger_value(&self) -> Result<bool> {
        self.inner.get_trigger_value().await.map_err(into_js_error)
    }

    #[wasm_bindgen(js_name = asCondition)]
    pub fn as_condition(&self) -> Condition {
        Condition {
            inner: dust_dds::dds_async::wait_set::ConditionAsync::StatusCondition(
                self.inner.clone(),
            ),
            entity_handle: self.entity_handle,
        }
    }

    #[wasm_bindgen(js_name = equals)]
    pub fn equals(&self, other: &StatusCondition) -> bool {
        self.entity_handle == other.entity_handle
    }
}

impl From<StatusCondition> for Condition {
    fn from(sc: StatusCondition) -> Self {
        sc.as_condition()
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Condition {
    pub(crate) inner: dust_dds::dds_async::wait_set::ConditionAsync,
    pub(crate) entity_handle: dust_dds::infrastructure::instance::InstanceHandle,
}

impl PartialEq for Condition {
    fn eq(&self, other: &Self) -> bool {
        self.entity_handle == other.entity_handle
    }
}
impl Eq for Condition {}

#[wasm_bindgen]
impl Condition {
    #[wasm_bindgen(js_name = fromStatusCondition)]
    pub fn from_status_condition(sc: &StatusCondition) -> Condition {
        sc.as_condition()
    }

    #[wasm_bindgen(js_name = getTriggerValue)]
    pub async fn get_trigger_value(&self) -> Result<bool> {
        self.inner.get_trigger_value().await.map_err(into_js_error)
    }

    #[wasm_bindgen(js_name = asStatusCondition)]
    pub fn as_status_condition(&self) -> Option<StatusCondition> {
        match &self.inner {
            dust_dds::dds_async::wait_set::ConditionAsync::StatusCondition(sc) => {
                Some(StatusCondition::new(sc.clone(), self.entity_handle))
            }
        }
    }

    #[wasm_bindgen(js_name = equals)]
    pub fn equals(&self, other: &Condition) -> bool {
        self.entity_handle == other.entity_handle
    }
}
