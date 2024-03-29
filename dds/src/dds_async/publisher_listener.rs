use std::{future::Future, pin::Pin};

use crate::{
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    publication::data_writer::AnyDataWriter,
};

/// This trait represents a listener object which can be associated with the [`PublisherAsync`](super::publisher::Publisher) entity.
pub trait PublisherListenerAsync {
    /// Method that is called when any writer belonging to this publisher reports a liveliness lost status.
    fn on_liveliness_lost<'a, 'b>(
        &'a mut self,
        _the_writer: &'b (dyn AnyDataWriter + Sync),
        _status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any writer belonging to this publisher reports an offered deadline missed status.
    fn on_offered_deadline_missed<'a, 'b>(
        &'a mut self,
        _the_writer: &'b (dyn AnyDataWriter + Sync),
        _status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any writer belonging to this publisher reports an offered incompatible qos status.
    fn on_offered_incompatible_qos<'a, 'b>(
        &'a mut self,
        _the_writer: &'b (dyn AnyDataWriter + Sync),
        _status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any writer belonging to this publisher reports a publication matched status.
    fn on_publication_matched<'a, 'b>(
        &'a mut self,
        _the_writer: &'b (dyn AnyDataWriter + Sync),
        _status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }
}
