use crate::dcps_psm::{
    LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
    PublicationMatchedStatus,
};

use super::data_writer::DataWriterProxy;

pub trait DataWriterListener {
    type Foo;

    fn on_liveliness_lost(
        &mut self,
        _the_writer: &DataWriterProxy<Self::Foo>,
        _status: LivelinessLostStatus,
    ) {
    }
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: &DataWriterProxy<Self::Foo>,
        _status: OfferedDeadlineMissedStatus,
    ) {
    }
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: &DataWriterProxy<Self::Foo>,
        _status: OfferedIncompatibleQosStatus,
    ) {
    }
    fn on_publication_matched(
        &mut self,
        _the_writer: &DataWriterProxy<Self::Foo>,
        _status: PublicationMatchedStatus,
    ) {
    }
}
