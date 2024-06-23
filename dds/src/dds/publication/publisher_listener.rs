use std::{future::Future, pin::Pin};

use crate::{
    dds_async::publisher_listener::PublisherListenerAsync,
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
};

use super::data_writer::AnyDataWriter;

/// This trait represents a listener object which can be associated with the [`Publisher`](super::publisher::Publisher) entity.
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

impl PublisherListenerAsync for Box<dyn PublisherListener + Send> {
    fn on_liveliness_lost<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        PublisherListener::on_liveliness_lost(self.as_mut(), the_writer, status);
        Box::pin(std::future::ready(()))
    }

    fn on_offered_deadline_missed<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        PublisherListener::on_offered_deadline_missed(self.as_mut(), the_writer, status);
        Box::pin(std::future::ready(()))
    }

    fn on_offered_incompatible_qos<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        PublisherListener::on_offered_incompatible_qos(self.as_mut(), the_writer, status);
        Box::pin(std::future::ready(()))
    }

    fn on_publication_matched<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        PublisherListener::on_publication_matched(self.as_mut(), the_writer, status);
        Box::pin(std::future::ready(()))
    }
}
