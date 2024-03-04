use dust_dds_derive::actor_interface;

use crate::{
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
    publication::publisher_listener::PublisherListener,
};

pub struct PublisherListenerActor {
    listener: Box<dyn PublisherListener + Send>,
}

impl PublisherListenerActor {
    pub fn new(listener: Box<dyn PublisherListener + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl PublisherListenerActor {
    async fn trigger_on_offered_incompatible_qos(&mut self, status: OfferedIncompatibleQosStatus) {
        tokio::task::block_in_place(|| self.listener.on_offered_incompatible_qos(&(), status));
    }

    async fn trigger_on_publication_matched(&mut self, status: PublicationMatchedStatus) {
        tokio::task::block_in_place(|| self.listener.on_publication_matched(&(), status));
    }
}
