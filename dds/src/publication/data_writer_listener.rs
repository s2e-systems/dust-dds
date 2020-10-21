use std::any::Any;
use crate::types::DDSType;
use crate::infrastructure::status::{LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus};
use crate::publication::data_writer::DataWriter;
use crate::infrastructure::listener::NoListener;

pub trait DataWriterListener<T: DDSType> {
    fn on_liveliness_lost(&self, the_writer: DataWriter<T>, status: LivelinessLostStatus);
    fn on_offered_deadline_missed(&self, the_writer: DataWriter<T>, status: OfferedDeadlineMissedStatus);
    fn on_offered_incompatible_qos(&self, the_writer: DataWriter<T>, status: OfferedIncompatibleQosStatus);
    fn on_publication_matched(&self, the_writer: DataWriter<T>, status: PublicationMatchedStatus);
}

impl<T: DDSType> DataWriterListener<T> for NoListener {
    fn on_liveliness_lost(&self, _the_writer: DataWriter<T>, _status: LivelinessLostStatus) {
        todo!()
    }

    fn on_offered_deadline_missed(&self, _the_writer: DataWriter<T>, _status: OfferedDeadlineMissedStatus) {
        todo!()
    }

    fn on_offered_incompatible_qos(&self, _the_writer: DataWriter<T>, _status: OfferedIncompatibleQosStatus) {
        todo!()
    }

    fn on_publication_matched(&self, _the_writer: DataWriter<T>, _status: PublicationMatchedStatus) {
        todo!()
    }
}