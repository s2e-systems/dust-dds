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
    fn on_liveliness_lost(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    fn on_publication_matched(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

impl<T> PublisherListenerAsyncDyn for T
where
    T: PublisherListenerAsync,
{
    fn on_liveliness_lost(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }

    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }

    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
    }

    fn on_publication_matched(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        todo!()
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
