use crate::dcps_psm::{
    LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
    PublicationMatchedStatus,
};

pub trait DataWriterListener {
    fn on_liveliness_lost(&self, _status: LivelinessLostStatus) {}
    fn on_offered_deadline_missed(&self, _status: OfferedDeadlineMissedStatus) {}
    fn on_offered_incompatible_qos(&self, _status: OfferedIncompatibleQosStatus) {}
    fn on_publication_matched(&self, _status: PublicationMatchedStatus) {}
}
