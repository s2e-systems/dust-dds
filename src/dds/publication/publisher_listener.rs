use crate::dds::infrastructure::status::{LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus};
use crate::dds::publication::data_writer::AnyDataWriter;
use crate::dds::infrastructure::listener::Listener;
pub trait PublisherListener : Listener {
    fn on_liveliness_lost(&self, the_writer: &dyn AnyDataWriter, status: LivelinessLostStatus);
    fn on_offered_deadline_missed(&self, the_writer: &dyn AnyDataWriter, status: OfferedDeadlineMissedStatus);
    fn on_offered_incompatible_qos(&self, the_writer: &dyn AnyDataWriter, status: OfferedIncompatibleQosStatus);
    fn on_publication_matched(&self, the_writer: &dyn AnyDataWriter, status: PublicationMatchedStatus);
}