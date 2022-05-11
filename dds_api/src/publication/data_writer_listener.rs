use crate::dcps_psm::{
    LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
    PublicationMatchedStatus,
};

pub trait DataWriterListener {
    fn on_liveliness_lost(&mut self, _status: LivelinessLostStatus) {}
    fn on_offered_deadline_missed(&mut self, _status: OfferedDeadlineMissedStatus) {}
    fn on_offered_incompatible_qos(&mut self, _status: OfferedIncompatibleQosStatus) {}
    fn on_publication_matched(&mut self, _status: PublicationMatchedStatus) {}
}
