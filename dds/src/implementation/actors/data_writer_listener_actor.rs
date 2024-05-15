use dust_dds_derive::actor_interface;

use crate::{
    dds_async::{publisher::PublisherAsync, topic::TopicAsync},
    implementation::actor::ActorWeakAddress,
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
};

use super::{
    any_data_writer_listener::AnyDataWriterListener, data_writer_actor::DataWriterActor,
    status_condition_actor::StatusConditionActor,
};

pub struct DataWriterListenerActor {
    listener: Option<Box<dyn AnyDataWriterListener + Send>>,
}

impl DataWriterListenerActor {
    pub fn new(listener: Option<Box<dyn AnyDataWriterListener + Send>>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl DataWriterListenerActor {
    async fn trigger_on_offered_incompatible_qos(
        &mut self,
        writer_address: ActorWeakAddress<DataWriterActor>,
        status_condition_address: ActorWeakAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: OfferedIncompatibleQosStatus,
    ) {
        if let Some(l) = &mut self.listener {
            l.trigger_on_offered_incompatible_qos(
                writer_address,
                status_condition_address,
                publisher,
                topic,
                status,
            )
            .await
        }
    }

    async fn trigger_on_publication_matched(
        &mut self,
        writer_address: ActorWeakAddress<DataWriterActor>,
        status_condition_address: ActorWeakAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: PublicationMatchedStatus,
    ) {
        if let Some(l) = &mut self.listener {
            l.trigger_on_publication_matched(
                writer_address,
                status_condition_address,
                publisher,
                topic,
                status,
            )
            .await
        }
    }
}
