use std::{future::Future, pin::Pin};

use dust_dds_derive::actor_interface;

use crate::{
    dds_async::publisher_listener::PublisherListenerAsync,
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    publication::data_writer::AnyDataWriter,
};

pub trait PublisherListenerAsyncDyn {
    fn on_liveliness_lost<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_offered_deadline_missed<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_offered_incompatible_qos<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_publication_matched<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;
}

impl<T> PublisherListenerAsyncDyn for T
where
    T: PublisherListenerAsync + Send,
{
    fn on_liveliness_lost<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_liveliness_lost(the_writer, status).await })
    }

    fn on_offered_deadline_missed<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_offered_deadline_missed(the_writer, status).await })
    }

    fn on_offered_incompatible_qos<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_offered_incompatible_qos(the_writer, status).await })
    }

    fn on_publication_matched<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_publication_matched(the_writer, status).await })
    }
}

pub struct PublisherListenerActor {
    listener: Box<dyn PublisherListenerAsyncDyn + Send>,
}

impl PublisherListenerActor {
    pub fn new(listener: Box<dyn PublisherListenerAsyncDyn + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl PublisherListenerActor {
    async fn trigger_on_offered_incompatible_qos(&mut self, status: OfferedIncompatibleQosStatus) {
        self.listener.on_offered_incompatible_qos(&(), status).await
    }

    async fn trigger_on_publication_matched(&mut self, status: PublicationMatchedStatus) {
        self.listener.on_publication_matched(&(), status).await
    }
}
