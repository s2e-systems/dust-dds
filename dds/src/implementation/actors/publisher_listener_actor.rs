use dust_dds_derive::actor_interface;

use crate::{
    dds_async::publisher_listener::PublisherListenerAsync,
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
};

pub struct PublisherListenerActor {
    listener: Box<dyn PublisherListenerAsync + Send>,
}

impl PublisherListenerActor {
    pub fn new(listener: Box<dyn PublisherListenerAsync + Send>) -> Self {
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
