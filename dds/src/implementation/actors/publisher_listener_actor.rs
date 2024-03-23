use dust_dds_derive::actor_interface;

use crate::{
    dds_async::publisher_listener::PublisherListenerAsync,
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
};

pub struct PublisherListenerActor {
    listener: Option<Box<dyn PublisherListenerAsync + Send>>,
}

impl PublisherListenerActor {
    pub fn new(listener: Option<Box<dyn PublisherListenerAsync + Send>>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl PublisherListenerActor {
    async fn trigger_on_offered_incompatible_qos(&mut self, status: OfferedIncompatibleQosStatus) {
        if let Some(l) = &mut self.listener {
            l.on_offered_incompatible_qos(&(), status).await
        }
    }

    async fn trigger_on_publication_matched(&mut self, status: PublicationMatchedStatus) {
        if let Some(l) = &mut self.listener {
            l.on_publication_matched(&(), status).await
        }
    }
}
