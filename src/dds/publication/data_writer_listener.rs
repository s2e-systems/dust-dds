use std::any::Any;
use crate::dds::infrastructure::status::{LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus};
use crate::dds::publication::data_writer::DataWriter;
use crate::dds::infrastructure::listener::NoListener;

pub trait DataWriterListener<T: Any+Send+Sync+std::fmt::Debug> : Any + Send + Sync {
    fn on_liveliness_lost(&self, the_writer: DataWriter<T>, status: LivelinessLostStatus);
    fn on_offered_deadline_missed(&self, the_writer: DataWriter<T>, status: OfferedDeadlineMissedStatus);
    fn on_offered_incompatible_qos(&self, the_writer: DataWriter<T>, status: OfferedIncompatibleQosStatus);
    fn on_publication_matched(&self, the_writer: DataWriter<T>, status: PublicationMatchedStatus);
}

impl<T: Any+Send+Sync+std::fmt::Debug> DataWriterListener<T> for NoListener {
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