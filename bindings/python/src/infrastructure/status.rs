use pyo3::prelude::*;

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
