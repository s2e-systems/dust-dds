use crate::types::DDSType;
use crate::dds_infrastructure::status::LivelinessLostStatus;
// use crate::infrastructure::status::{LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus};
// use crate::publication::data_writer::DataWriter;
// use crate::infrastructure::qos::DataWriterQos;
use crate::dds::publication::data_writer::DataWriter;

pub trait DataWriterListener<T: DDSType> {
    fn on_liveliness_lost(&self, the_writer: DataWriter<T>, status: LivelinessLostStatus);
    // fn on_offered_deadline_missed<T: DDSType>(&self, the_writer:  dyn DataWriter<T>, status: OfferedDeadlineMissedStatus);
    // fn on_offered_incompatible_qos<T: DDSType>(&self, the_writer:  dyn DataWriter<T>, status: OfferedIncompatibleQosStatus);
    // fn on_publication_matched<T: DDSType>(&self, the_writer:  dyn DataWriter<T>, status: PublicationMatchedStatus);
}