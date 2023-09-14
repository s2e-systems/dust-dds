use crate::infrastructure::status::{
    LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
    PublicationMatchedStatus,
};

use super::data_writer::DataWriter;

pub trait DataWriterListener<Foo> {
    fn on_liveliness_lost(&mut self, _the_writer: &DataWriter<Foo>, _status: LivelinessLostStatus) {
    }
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: &DataWriter<Foo>,
        _status: OfferedDeadlineMissedStatus,
    ) {
    }
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: &DataWriter<Foo>,
        _status: OfferedIncompatibleQosStatus,
    ) {
    }
    fn on_publication_matched(
        &mut self,
        _the_writer: &DataWriter<Foo>,
        _status: PublicationMatchedStatus,
    ) {
    }
}
