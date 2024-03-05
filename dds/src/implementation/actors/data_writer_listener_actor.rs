use dust_dds_derive::actor_interface;

use crate::{
    dds_async::{publisher::PublisherAsync, topic::TopicAsync},
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
};

use super::{
    any_data_writer_listener::AnyDataWriterListener, data_writer_actor::DataWriterActor,
    status_condition_actor::StatusConditionActor,
};

pub struct DataWriterListenerActor {
    listener: Box<dyn AnyDataWriterListener + Send + 'static>,
}

impl DataWriterListenerActor {
    pub fn new(listener: Box<dyn AnyDataWriterListener + Send + 'static>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl DataWriterListenerActor {
    async fn trigger_on_offered_incompatible_qos(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: OfferedIncompatibleQosStatus,
    ) {
        self.listener
            .trigger_on_offered_incompatible_qos(
                writer_address,
                status_condition_address,
                publisher,
                topic,
                status,
            )
            .await
    }

    async fn trigger_on_publication_matched(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: PublicationMatchedStatus,
    ) {
        self.listener
            .trigger_on_publication_matched(
                writer_address,
                status_condition_address,
                publisher,
                topic,
                status,
            )
            .await
    }
}
