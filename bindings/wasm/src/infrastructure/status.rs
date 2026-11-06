use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicationMatchedStatus {
    pub(crate) inner: dust_dds::infrastructure::status::PublicationMatchedStatus,
}

#[wasm_bindgen]
impl PublicationMatchedStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }

    #[wasm_bindgen(getter, js_name = currentCount)]
    pub fn current_count(&self) -> i32 {
        self.inner.current_count
    }

    #[wasm_bindgen(getter, js_name = currentCountChange)]
    pub fn current_count_change(&self) -> i32 {
        self.inner.current_count_change
    }
}

impl From<dust_dds::infrastructure::status::PublicationMatchedStatus> for PublicationMatchedStatus {
    fn from(inner: dust_dds::infrastructure::status::PublicationMatchedStatus) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubscriptionMatchedStatus {
    pub(crate) inner: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
}

#[wasm_bindgen]
impl SubscriptionMatchedStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }

    #[wasm_bindgen(getter, js_name = currentCount)]
    pub fn current_count(&self) -> i32 {
        self.inner.current_count
    }

    #[wasm_bindgen(getter, js_name = currentCountChange)]
    pub fn current_count_change(&self) -> i32 {
        self.inner.current_count_change
    }
}

impl From<dust_dds::infrastructure::status::SubscriptionMatchedStatus>
    for SubscriptionMatchedStatus
{
    fn from(inner: dust_dds::infrastructure::status::SubscriptionMatchedStatus) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    InconsistentTopic,
    OfferedDeadlineMissed,
    RequestedDeadlineMissed,
    OfferedIncompatibleQos,
    RequestedIncompatibleQos,
    SampleLost,
    SampleRejected,
    DataOnReaders,
    DataAvailable,
    LivelinessLost,
    LivelinessChanged,
    PublicationMatched,
    SubscriptionMatched,
}

impl From<dust_dds::infrastructure::status::StatusKind> for StatusKind {
    fn from(value: dust_dds::infrastructure::status::StatusKind) -> Self {
        match value {
            dust_dds::infrastructure::status::StatusKind::InconsistentTopic => {
                StatusKind::InconsistentTopic
            }
            dust_dds::infrastructure::status::StatusKind::OfferedDeadlineMissed => {
                StatusKind::OfferedDeadlineMissed
            }
            dust_dds::infrastructure::status::StatusKind::RequestedDeadlineMissed => {
                StatusKind::RequestedDeadlineMissed
            }
            dust_dds::infrastructure::status::StatusKind::OfferedIncompatibleQos => {
                StatusKind::OfferedIncompatibleQos
            }
            dust_dds::infrastructure::status::StatusKind::RequestedIncompatibleQos => {
                StatusKind::RequestedIncompatibleQos
            }
            dust_dds::infrastructure::status::StatusKind::SampleLost => StatusKind::SampleLost,
            dust_dds::infrastructure::status::StatusKind::SampleRejected => {
                StatusKind::SampleRejected
            }
            dust_dds::infrastructure::status::StatusKind::DataOnReaders => {
                StatusKind::DataOnReaders
            }
            dust_dds::infrastructure::status::StatusKind::DataAvailable => {
                StatusKind::DataAvailable
            }
            dust_dds::infrastructure::status::StatusKind::LivelinessLost => {
                StatusKind::LivelinessLost
            }
            dust_dds::infrastructure::status::StatusKind::LivelinessChanged => {
                StatusKind::LivelinessChanged
            }
            dust_dds::infrastructure::status::StatusKind::PublicationMatched => {
                StatusKind::PublicationMatched
            }
            dust_dds::infrastructure::status::StatusKind::SubscriptionMatched => {
                StatusKind::SubscriptionMatched
            }
        }
    }
}

impl From<StatusKind> for dust_dds::infrastructure::status::StatusKind {
    fn from(value: StatusKind) -> Self {
        match value {
            StatusKind::InconsistentTopic => {
                dust_dds::infrastructure::status::StatusKind::InconsistentTopic
            }
            StatusKind::OfferedDeadlineMissed => {
                dust_dds::infrastructure::status::StatusKind::OfferedDeadlineMissed
            }
            StatusKind::RequestedDeadlineMissed => {
                dust_dds::infrastructure::status::StatusKind::RequestedDeadlineMissed
            }
            StatusKind::OfferedIncompatibleQos => {
                dust_dds::infrastructure::status::StatusKind::OfferedIncompatibleQos
            }
            StatusKind::RequestedIncompatibleQos => {
                dust_dds::infrastructure::status::StatusKind::RequestedIncompatibleQos
            }
            StatusKind::SampleLost => dust_dds::infrastructure::status::StatusKind::SampleLost,
            StatusKind::SampleRejected => {
                dust_dds::infrastructure::status::StatusKind::SampleRejected
            }
            StatusKind::DataOnReaders => {
                dust_dds::infrastructure::status::StatusKind::DataOnReaders
            }
            StatusKind::DataAvailable => {
                dust_dds::infrastructure::status::StatusKind::DataAvailable
            }
            StatusKind::LivelinessLost => {
                dust_dds::infrastructure::status::StatusKind::LivelinessLost
            }
            StatusKind::LivelinessChanged => {
                dust_dds::infrastructure::status::StatusKind::LivelinessChanged
            }
            StatusKind::PublicationMatched => {
                dust_dds::infrastructure::status::StatusKind::PublicationMatched
            }
            StatusKind::SubscriptionMatched => {
                dust_dds::infrastructure::status::StatusKind::SubscriptionMatched
            }
        }
    }
}
