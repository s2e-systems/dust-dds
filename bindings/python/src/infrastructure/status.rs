use pyo3::prelude::*;

use super::instance::InstanceHandle;

#[pyclass]
#[derive(Clone)]
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

#[pyclass]
#[derive(Clone)]
pub struct InconsistentTopicStatus(dust_dds::infrastructure::status::InconsistentTopicStatus);

impl From<dust_dds::infrastructure::status::InconsistentTopicStatus> for InconsistentTopicStatus {
    fn from(value: dust_dds::infrastructure::status::InconsistentTopicStatus) -> Self {
        Self(value)
    }
}

#[pymethods]
impl InconsistentTopicStatus {
    pub fn get_total_count(&self) -> i32 {
        self.0.total_count
    }

    pub fn get_total_count_change(&self) -> i32 {
        self.0.total_count_change
    }
}

#[pyclass]
#[derive(Clone)]
pub struct SampleLostStatus(dust_dds::infrastructure::status::SampleLostStatus);

impl From<dust_dds::infrastructure::status::SampleLostStatus> for SampleLostStatus {
    fn from(value: dust_dds::infrastructure::status::SampleLostStatus) -> Self {
        Self(value)
    }
}

#[pymethods]
impl SampleLostStatus {
    pub fn get_total_count(&self) -> i32 {
        self.0.total_count
    }

    pub fn get_total_count_change(&self) -> i32 {
        self.0.total_count_change
    }
}

#[pyclass]
#[derive(Clone)]
pub enum SampleRejectedStatusKind {
    NotRejected,
    RejectedByInstancesLimit,
    RejectedBySamplesLimit,
    RejectedBySamplesPerInstanceLimit,
}

impl From<dust_dds::infrastructure::status::SampleRejectedStatusKind> for SampleRejectedStatusKind {
    fn from(value: dust_dds::infrastructure::status::SampleRejectedStatusKind) -> Self {
        match value {
            dust_dds::infrastructure::status::SampleRejectedStatusKind::NotRejected => SampleRejectedStatusKind::NotRejected,
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedByInstancesLimit => SampleRejectedStatusKind::RejectedByInstancesLimit,
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedBySamplesLimit => SampleRejectedStatusKind::RejectedBySamplesLimit,
            dust_dds::infrastructure::status::SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit => SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct SampleRejectedStatus(dust_dds::infrastructure::status::SampleRejectedStatus);

impl From<dust_dds::infrastructure::status::SampleRejectedStatus> for SampleRejectedStatus {
    fn from(value: dust_dds::infrastructure::status::SampleRejectedStatus) -> Self {
        Self(value)
    }
}

#[pymethods]
impl SampleRejectedStatus {
    pub fn get_total_count(&self) -> i32 {
        self.0.total_count
    }

    pub fn get_total_count_change(&self) -> i32 {
        self.0.total_count_change
    }

    pub fn get_last_reason(&self) -> SampleRejectedStatusKind {
        self.0.last_reason.into()
    }

    pub fn get_last_instance_handle(&self) -> InstanceHandle {
        self.0.last_instance_handle.into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct LivelinessLostStatus(dust_dds::infrastructure::status::LivelinessLostStatus);

impl From<dust_dds::infrastructure::status::LivelinessLostStatus> for LivelinessLostStatus {
    fn from(value: dust_dds::infrastructure::status::LivelinessLostStatus) -> Self {
        Self(value)
    }
}

#[pymethods]
impl LivelinessLostStatus {
    pub fn get_total_count(&self) -> i32 {
        self.0.total_count
    }

    pub fn get_total_count_change(&self) -> i32 {
        self.0.total_count_change
    }
}

#[pyclass]
#[derive(Clone)]
pub struct LivelinessChangedStatus(dust_dds::infrastructure::status::LivelinessChangedStatus);

impl From<dust_dds::infrastructure::status::LivelinessChangedStatus> for LivelinessChangedStatus {
    fn from(value: dust_dds::infrastructure::status::LivelinessChangedStatus) -> Self {
        Self(value)
    }
}

#[pymethods]
impl LivelinessChangedStatus {
    pub fn get_alive_count(&self) -> i32 {
        self.0.alive_count
    }

    pub fn get_not_alive_count(&self) -> i32 {
        self.0.not_alive_count
    }

    pub fn get_alive_count_change(&self) -> i32 {
        self.0.alive_count_change
    }

    pub fn get_not_alive_count_change(&self) -> i32 {
        self.0.not_alive_count_change
    }

    pub fn get_last_publication_handle(&self) -> InstanceHandle {
        self.0.last_publication_handle.into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct OfferedDeadlineMissedStatus(
    dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
);

impl From<dust_dds::infrastructure::status::OfferedDeadlineMissedStatus>
    for OfferedDeadlineMissedStatus
{
    fn from(value: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus) -> Self {
        Self(value)
    }
}

#[pymethods]
impl OfferedDeadlineMissedStatus {
    pub fn get_total_count(&self) -> i32 {
        self.0.total_count
    }

    pub fn get_total_count_change(&self) -> i32 {
        self.0.total_count_change
    }

    pub fn get_last_instance_handle(&self) -> InstanceHandle {
        self.0.last_instance_handle.into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct RequestedDeadlineMissedStatus(
    dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
);

impl From<dust_dds::infrastructure::status::RequestedDeadlineMissedStatus>
    for RequestedDeadlineMissedStatus
{
    fn from(value: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus) -> Self {
        Self(value)
    }
}

#[pymethods]
impl RequestedDeadlineMissedStatus {
    pub fn get_total_count(&self) -> i32 {
        self.0.total_count
    }

    pub fn get_total_count_change(&self) -> i32 {
        self.0.total_count_change
    }

    pub fn get_last_instance_handle(&self) -> InstanceHandle {
        self.0.last_instance_handle.into()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct QosPolicyCount(dust_dds::infrastructure::status::QosPolicyCount);

impl From<dust_dds::infrastructure::status::QosPolicyCount> for QosPolicyCount {
    fn from(value: dust_dds::infrastructure::status::QosPolicyCount) -> Self {
        Self(value)
    }
}

#[pymethods]
impl QosPolicyCount {
    pub fn get_policy_id(&self) -> i32 {
        self.0.policy_id
    }

    pub fn get_count(&self) -> i32 {
        self.0.count
    }
}

#[pyclass]
#[derive(Clone)]
pub struct OfferedIncompatibleQosStatus(
    dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
);

impl From<dust_dds::infrastructure::status::OfferedIncompatibleQosStatus>
    for OfferedIncompatibleQosStatus
{
    fn from(value: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus) -> Self {
        Self(value)
    }
}

#[pymethods]
impl OfferedIncompatibleQosStatus {
    pub fn get_total_count(&self) -> i32 {
        self.0.total_count
    }

    pub fn get_total_count_change(&self) -> i32 {
        self.0.total_count_change
    }

    pub fn get_last_policy_id(&self) -> i32 {
        self.0.last_policy_id
    }

    pub fn get_policies(&self) -> Vec<QosPolicyCount> {
        self.0.policies.iter().map(|p| p.clone().into()).collect()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct RequestedIncompatibleQosStatus(
    dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
);

impl From<dust_dds::infrastructure::status::RequestedIncompatibleQosStatus>
    for RequestedIncompatibleQosStatus
{
    fn from(value: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus) -> Self {
        Self(value)
    }
}

#[pymethods]
impl RequestedIncompatibleQosStatus {
    pub fn get_total_count(&self) -> i32 {
        self.0.total_count
    }

    pub fn get_total_count_change(&self) -> i32 {
        self.0.total_count_change
    }

    pub fn get_last_policy_id(&self) -> i32 {
        self.0.last_policy_id
    }

    pub fn get_policies(&self) -> Vec<QosPolicyCount> {
        self.0.policies.iter().map(|p| p.clone().into()).collect()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PublicationMatchedStatus(dust_dds::infrastructure::status::PublicationMatchedStatus);

impl From<dust_dds::infrastructure::status::PublicationMatchedStatus> for PublicationMatchedStatus {
    fn from(value: dust_dds::infrastructure::status::PublicationMatchedStatus) -> Self {
        Self(value)
    }
}

#[pymethods]
impl PublicationMatchedStatus {
    pub fn get_total_count(&self) -> i32 {
        self.0.total_count
    }

    pub fn get_total_count_change(&self) -> i32 {
        self.0.total_count_change
    }

    pub fn get_last_subscription_handle(&self) -> InstanceHandle {
        self.0.last_subscription_handle.into()
    }

    pub fn get_current_count(&self) -> i32 {
        self.0.current_count
    }

    pub fn get_current_count_change(&self) -> i32 {
        self.0.current_count_change
    }
}

#[pyclass]
#[derive(Clone)]
pub struct SubscriptionMatchedStatus(dust_dds::infrastructure::status::SubscriptionMatchedStatus);

impl From<dust_dds::infrastructure::status::SubscriptionMatchedStatus>
    for SubscriptionMatchedStatus
{
    fn from(value: dust_dds::infrastructure::status::SubscriptionMatchedStatus) -> Self {
        Self(value)
    }
}

#[pymethods]
impl SubscriptionMatchedStatus {
    pub fn get_total_count(&self) -> i32 {
        self.0.total_count
    }

    pub fn get_total_count_change(&self) -> i32 {
        self.0.total_count_change
    }

    pub fn get_last_publication_handle(&self) -> InstanceHandle {
        self.0.last_publication_handle.into()
    }

    pub fn get_current_count(&self) -> i32 {
        self.0.current_count
    }

    pub fn get_current_count_change(&self) -> i32 {
        self.0.current_count_change
    }
}
