use std::any::Any;
use crate::dds::infrastructure::status::{LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus};
use crate::dds::publication::data_writer::DataWriter;

pub trait DataWriterListener : Any + Send + Sync {
    fn on_liveliness_lost(&self, the_writer: DataWriter, status: LivelinessLostStatus);
    fn on_offered_deadline_missed(&self, the_writer: DataWriter, status: OfferedDeadlineMissedStatus);
    fn on_offered_incompatible_qos(&self, the_writer: DataWriter, status: OfferedIncompatibleQosStatus);
    fn on_publication_matched(&self, the_writer: DataWriter, status: PublicationMatchedStatus);
}