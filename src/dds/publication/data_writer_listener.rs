use crate::types::DDSType;
use crate::dds::infrastructure::status::{LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus};
use crate::dds::infrastructure::listener::Listener;
use crate::dds::publication::data_writer::DataWriter;

pub trait DataWriterListener<T:DDSType> : Listener {
    // fn on_liveliness_lost(&self, the_writer: DataWriter<T>, status: LivelinessLostStatus) where Self:Sized;
    // fn on_offered_deadline_missed(&self, the_writer: DataWriter<T>, status: OfferedDeadlineMissedStatus) where Self:Sized;
    // fn on_offered_incompatible_qos(&self, the_writer: DataWriter<T>, status: OfferedIncompatibleQosStatus) where Self:Sized;
    // fn on_publication_matched(&self, the_writer: DataWriter<T>, status: PublicationMatchedStatus) where Self:Sized;
}