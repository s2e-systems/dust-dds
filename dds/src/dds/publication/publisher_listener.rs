use crate::infrastructure::status::{
    LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
    PublicationMatchedStatus,
};

use super::data_writer::AnyDataWriter;

/// This trait represents a listener object which can be associated with the ['Publisher'](super::publisher::Publisher) entity.
pub trait PublisherListener {
    /// Method that is called when any writer belonging to this publisher reports a liveliness lost status.
    fn on_liveliness_lost(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: LivelinessLostStatus,
    ) {
    }

    /// Method that is called when any writer belonging to this publisher reports an offered deadline missed status.
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: OfferedDeadlineMissedStatus,
    ) {
    }

    /// Method that is called when any writer belonging to this publisher reports an offered incompatible qos status.
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: OfferedIncompatibleQosStatus,
    ) {
    }

    /// Method that is called when any writer belonging to this publisher reports a publication matched status.
    fn on_publication_matched(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: PublicationMatchedStatus,
    ) {
    }
}
