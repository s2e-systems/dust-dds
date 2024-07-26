use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{data_writer::DataWriterAsync, publisher_listener::PublisherListenerAsync},
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
};

use super::data_writer::DataWriter;

/// This trait represents a listener object which can be associated with the [`Publisher`](super::publisher::Publisher) entity.
pub trait PublisherListener {
    /// Method that is called when any writer belonging to this publisher reports a liveliness lost status.
    fn on_liveliness_lost(&mut self, _the_writer: DataWriter<()>, _status: LivelinessLostStatus) {}

    /// Method that is called when any writer belonging to this publisher reports an offered deadline missed status.
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: DataWriter<()>,
        _status: OfferedDeadlineMissedStatus,
    ) {
    }

    /// Method that is called when any writer belonging to this publisher reports an offered incompatible qos status.
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: DataWriter<()>,
        _status: OfferedIncompatibleQosStatus,
    ) {
    }

    /// Method that is called when any writer belonging to this publisher reports a publication matched status.
    fn on_publication_matched(
        &mut self,
        _the_writer: DataWriter<()>,
        _status: PublicationMatchedStatus,
    ) {
    }
}

impl PublisherListenerAsync for Box<dyn PublisherListener + Send> {
    fn on_liveliness_lost(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        PublisherListener::on_liveliness_lost(self.as_mut(), DataWriter::new(the_writer), status);
        Box::pin(std::future::ready(()))
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        PublisherListener::on_offered_deadline_missed(
            self.as_mut(),
            DataWriter::new(the_writer),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        PublisherListener::on_offered_incompatible_qos(
            self.as_mut(),
            DataWriter::new(the_writer),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_publication_matched(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        PublisherListener::on_publication_matched(
            self.as_mut(),
            DataWriter::new(the_writer),
            status,
        );
        Box::pin(std::future::ready(()))
    }
}
