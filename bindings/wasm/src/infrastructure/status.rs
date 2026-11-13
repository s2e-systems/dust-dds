use wasm_bindgen::prelude::*;

use crate::infrastructure::instance::InstanceHandle;

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

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InconsistentTopicStatus {
    pub(crate) inner: dust_dds::infrastructure::status::InconsistentTopicStatus,
}

#[wasm_bindgen]
impl InconsistentTopicStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }
}

impl From<dust_dds::infrastructure::status::InconsistentTopicStatus> for InconsistentTopicStatus {
    fn from(inner: dust_dds::infrastructure::status::InconsistentTopicStatus) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SampleLostStatus {
    pub(crate) inner: dust_dds::infrastructure::status::SampleLostStatus,
}

#[wasm_bindgen]
impl SampleLostStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }
}

impl From<dust_dds::infrastructure::status::SampleLostStatus> for SampleLostStatus {
    fn from(inner: dust_dds::infrastructure::status::SampleLostStatus) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleRejectedStatusKind {
    NotRejected,
    RejectedByInstancesLimit,
    RejectedBySamplesLimit,
    RejectedBySamplesPerInstanceLimit,
}

impl From<dust_dds::infrastructure::status::SampleRejectedStatusKind> for SampleRejectedStatusKind {
    fn from(value: dust_dds::infrastructure::status::SampleRejectedStatusKind) -> Self {
        match value {
            dust_dds::infrastructure::status::SampleRejectedStatusKind::NotRejected => {
                SampleRejectedStatusKind::NotRejected
            }
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedByInstancesLimit => {
                SampleRejectedStatusKind::RejectedByInstancesLimit
            }
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedBySamplesLimit => {
                SampleRejectedStatusKind::RejectedBySamplesLimit
            }
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit => {
                SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit
            }
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SampleRejectedStatus {
    pub(crate) inner: dust_dds::infrastructure::status::SampleRejectedStatus,
}

#[wasm_bindgen]
impl SampleRejectedStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }

    #[wasm_bindgen(getter, js_name = lastReason)]
    pub fn last_reason(&self) -> SampleRejectedStatusKind {
        self.inner.last_reason.into()
    }

    #[wasm_bindgen(getter, js_name = lastInstanceHandle)]
    pub fn last_instance_handle(&self) -> InstanceHandle {
        self.inner.last_instance_handle.into()
    }
}

impl From<dust_dds::infrastructure::status::SampleRejectedStatus> for SampleRejectedStatus {
    fn from(inner: dust_dds::infrastructure::status::SampleRejectedStatus) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivelinessLostStatus {
    pub(crate) inner: dust_dds::infrastructure::status::LivelinessLostStatus,
}

#[wasm_bindgen]
impl LivelinessLostStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }
}

impl From<dust_dds::infrastructure::status::LivelinessLostStatus> for LivelinessLostStatus {
    fn from(inner: dust_dds::infrastructure::status::LivelinessLostStatus) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivelinessChangedStatus {
    pub(crate) inner: dust_dds::infrastructure::status::LivelinessChangedStatus,
}

#[wasm_bindgen]
impl LivelinessChangedStatus {
    #[wasm_bindgen(getter, js_name = aliveCount)]
    pub fn alive_count(&self) -> i32 {
        self.inner.alive_count
    }

    #[wasm_bindgen(getter, js_name = notAliveCount)]
    pub fn not_alive_count(&self) -> i32 {
        self.inner.not_alive_count
    }

    #[wasm_bindgen(getter, js_name = aliveCountChange)]
    pub fn alive_count_change(&self) -> i32 {
        self.inner.alive_count_change
    }

    #[wasm_bindgen(getter, js_name = notAliveCountChange)]
    pub fn not_alive_count_change(&self) -> i32 {
        self.inner.not_alive_count_change
    }

    #[wasm_bindgen(getter, js_name = lastPublicationHandle)]
    pub fn last_publication_handle(&self) -> InstanceHandle {
        self.inner.last_publication_handle.into()
    }
}

impl From<dust_dds::infrastructure::status::LivelinessChangedStatus> for LivelinessChangedStatus {
    fn from(inner: dust_dds::infrastructure::status::LivelinessChangedStatus) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferedDeadlineMissedStatus {
    pub(crate) inner: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
}

#[wasm_bindgen]
impl OfferedDeadlineMissedStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }

    #[wasm_bindgen(getter, js_name = lastInstanceHandle)]
    pub fn last_instance_handle(&self) -> InstanceHandle {
        self.inner.last_instance_handle.into()
    }
}

impl From<dust_dds::infrastructure::status::OfferedDeadlineMissedStatus>
    for OfferedDeadlineMissedStatus
{
    fn from(inner: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestedDeadlineMissedStatus {
    pub(crate) inner: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
}

#[wasm_bindgen]
impl RequestedDeadlineMissedStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }

    #[wasm_bindgen(getter, js_name = lastInstanceHandle)]
    pub fn last_instance_handle(&self) -> InstanceHandle {
        self.inner.last_instance_handle.into()
    }
}

impl From<dust_dds::infrastructure::status::RequestedDeadlineMissedStatus>
    for RequestedDeadlineMissedStatus
{
    fn from(inner: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QosPolicyCount {
    pub(crate) inner: dust_dds::infrastructure::status::QosPolicyCount,
}

#[wasm_bindgen]
impl QosPolicyCount {
    #[wasm_bindgen(getter, js_name = policyId)]
    pub fn policy_id(&self) -> i32 {
        self.inner.policy_id
    }

    #[wasm_bindgen(getter, js_name = count)]
    pub fn count(&self) -> i32 {
        self.inner.count
    }
}

impl From<dust_dds::infrastructure::status::QosPolicyCount> for QosPolicyCount {
    fn from(inner: dust_dds::infrastructure::status::QosPolicyCount) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferedIncompatibleQosStatus {
    pub(crate) inner: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
}

#[wasm_bindgen]
impl OfferedIncompatibleQosStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }

    #[wasm_bindgen(getter, js_name = lastPolicyId)]
    pub fn last_policy_id(&self) -> i32 {
        self.inner.last_policy_id
    }

    #[wasm_bindgen(getter, js_name = policies)]
    pub fn policies(&self) -> js_sys::Array {
        let array = js_sys::Array::new();
        for p in &self.inner.policies {
            array.push(&JsValue::from(QosPolicyCount::from(p.clone())));
        }
        array
    }
}

impl From<dust_dds::infrastructure::status::OfferedIncompatibleQosStatus>
    for OfferedIncompatibleQosStatus
{
    fn from(inner: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestedIncompatibleQosStatus {
    pub(crate) inner: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
}

#[wasm_bindgen]
impl RequestedIncompatibleQosStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }

    #[wasm_bindgen(getter, js_name = lastPolicyId)]
    pub fn last_policy_id(&self) -> i32 {
        self.inner.last_policy_id
    }

    #[wasm_bindgen(getter, js_name = policies)]
    pub fn policies(&self) -> js_sys::Array {
        let array = js_sys::Array::new();
        for p in &self.inner.policies {
            array.push(&JsValue::from(QosPolicyCount::from(p.clone())));
        }
        array
    }
}

impl From<dust_dds::infrastructure::status::RequestedIncompatibleQosStatus>
    for RequestedIncompatibleQosStatus
{
    fn from(inner: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus) -> Self {
        Self { inner }
    }
}
