use crate::dcps_psm::{
    LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
    PublicationMatchedStatus,
};

pub trait DataWriterListener {
    type DataType;
    fn on_liveliness_lost(&self, status: LivelinessLostStatus);
    fn on_offered_deadline_missed(&self, status: OfferedDeadlineMissedStatus);
    fn on_offered_incompatible_qos(&self, status: OfferedIncompatibleQosStatus);
    fn on_publication_matched(&self, status: PublicationMatchedStatus);
}
