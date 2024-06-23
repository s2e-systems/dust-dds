use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{data_writer::DataWriterAsync, data_writer_listener::DataWriterListenerAsync},
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
};

use super::data_writer::DataWriter;

/// This trait represents a listener object which can be associated with the [`DataWriter`] entity.
pub trait DataWriterListener<'a>: 'static {
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

impl<'a, Foo> DataWriterListenerAsync<'_> for Box<dyn DataWriterListener<'_, Foo = Foo> + Send + 'a>
where
    Self: 'static,
{
    type Foo = Foo;

    fn on_liveliness_lost(
        &mut self,
        the_writer: DataWriterAsync<Self::Foo>,
        status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DataWriterListener::on_liveliness_lost(self.as_mut(), DataWriter::new(the_writer), status);
        Box::pin(std::future::ready(()))
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: DataWriterAsync<Self::Foo>,
        status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DataWriterListener::on_offered_deadline_missed(
            self.as_mut(),
            DataWriter::new(the_writer),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterAsync<Self::Foo>,
        status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DataWriterListener::on_offered_incompatible_qos(
            self.as_mut(),
            DataWriter::new(the_writer),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_publication_matched(
        &mut self,
        the_writer: DataWriterAsync<Self::Foo>,
        status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DataWriterListener::on_publication_matched(
            self.as_mut(),
            DataWriter::new(the_writer),
            status,
        );
        Box::pin(std::future::ready(()))
    }
}
