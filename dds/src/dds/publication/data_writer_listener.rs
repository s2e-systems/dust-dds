use crate::infrastructure::status::{
    LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
    PublicationMatchedStatus,
};

use super::data_writer::DataWriter;

/// This trait represents a listener object which can be associated with the [`DataWriter`] entity.
pub trait DataWriterListener {
    /// Type of the DataWriter with which this Listener will be associated.
    type Foo;

    /// Method that is called when this writer reports a liveliness lost status.
    fn on_liveliness_lost(
        &mut self,
        _the_writer: DataWriter<Self::Foo>,
        _status: LivelinessLostStatus,
    ) {
    }
    /// Method that is called when this writer reports an offered deadline missed status.
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: DataWriter<Self::Foo>,
        _status: OfferedDeadlineMissedStatus,
    ) {
    }
    /// Method that is called when this writer reports an offered incompatible qos status.
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: DataWriter<Self::Foo>,
        _status: OfferedIncompatibleQosStatus,
    ) {
    }
    /// Method that is called when this writer reports a publication matched status.
    fn on_publication_matched(
        &mut self,
        _the_writer: DataWriter<Self::Foo>,
        _status: PublicationMatchedStatus,
    ) {
    }
}
