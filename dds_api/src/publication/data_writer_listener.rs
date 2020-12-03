use crate::types::DDSType;
use crate::infrastructure::status::{LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus};
use crate::publication::data_writer::DataWriter;
use crate::infrastructure::qos::DataWriterQos;

pub trait DataWriterListener<T: DDSType> {
    fn on_liveliness_lost(&self, the_writer: dyn DataWriter<T>, status: LivelinessLostStatus);
    fn on_offered_deadline_missed(&self, the_writer:  dyn DataWriter<T>, status: OfferedDeadlineMissedStatus);
    fn on_offered_incompatible_qos(&self, the_writer:  dyn DataWriter<T>, status: OfferedIncompatibleQosStatus);
    fn on_publication_matched(&self, the_writer:  dyn DataWriter<T>, status: PublicationMatchedStatus);
}