use dust_dds_derive::actor_interface;

use crate::{
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
};

use super::{
    any_data_writer_listener::AnyDataWriterListener, data_writer_actor::DataWriterActor,
    domain_participant_actor::DomainParticipantActor, publisher_actor::PublisherActor,
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
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
        status: OfferedIncompatibleQosStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_offered_incompatible_qos(
                writer_address,
                publisher_address,
                participant_address,
                runtime_handle,
                status,
            )
        });
    }

    async fn trigger_on_publication_matched(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
        status: PublicationMatchedStatus,
    ) {
        tokio::task::block_in_place(|| {
            self.listener.trigger_on_publication_matched(
                writer_address,
                publisher_address,
                participant_address,
                runtime_handle,
                status,
            )
        });
    }
}
