use std::{future::Future, pin::Pin};

use crate::infrastructure::status::{
    LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
    PublicationMatchedStatus,
};

use super::data_writer::DataWriterAsync;

/// This trait represents a listener object which can be associated with the [`DataWriterAsync`] entity.
pub trait DataWriterListenerAsync<'a>
where
    Self: 'static,
{
    /// Type of the DataWriter with which this Listener will be associated.
    type Foo;

    /// Method that is called when this writer reports a liveliness lost status.
    fn on_liveliness_lost(
        &mut self,
        _the_writer: DataWriterAsync<Self::Foo>,
        _status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }
    /// Method that is called when this writer reports an offered deadline missed status.
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: DataWriterAsync<Self::Foo>,
        _status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }
    /// Method that is called when this writer reports an offered incompatible qos status.
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: DataWriterAsync<Self::Foo>,
        _status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }
    /// Method that is called when this writer reports a publication matched status.
    fn on_publication_matched(
        &mut self,
        _the_writer: DataWriterAsync<Self::Foo>,
        _status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }
}
