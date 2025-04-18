use super::{entities::domain_participant::DomainParticipantEntity, handle::InstanceHandleCounter};
use crate::{
    dds_async::{
        data_reader::DataReaderAsync, data_writer::DataWriterAsync,
        domain_participant::DomainParticipantAsync, publisher::PublisherAsync,
        subscriber::SubscriberAsync, topic::TopicAsync,
    },
    implementation::domain_participant_factory::domain_participant_factory_actor::DdsTransportParticipant,
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
    },
    runtime::{actor::ActorAddress, executor::Executor, timer::TimerDriver},
};

pub struct DomainParticipantActor {
    pub transport: DdsTransportParticipant,
    pub instance_handle_counter: InstanceHandleCounter,
    pub entity_counter: u16,
    pub domain_participant: DomainParticipantEntity,
    pub backend_executor: Executor,
    pub listener_executor: Executor,
    pub timer_driver: TimerDriver,
}

impl DomainParticipantActor {
    pub fn new(
        domain_participant: DomainParticipantEntity,
        transport: DdsTransportParticipant,
        backend_executor: Executor,
        listener_executor: Executor,
        timer_driver: TimerDriver,
        instance_handle_counter: InstanceHandleCounter,
    ) -> Self {
        Self {
            transport,
            instance_handle_counter,
            entity_counter: 0,
            domain_participant,
            backend_executor,
            listener_executor,
            timer_driver,
        }
    }

    pub fn get_participant_async(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> DomainParticipantAsync {
        DomainParticipantAsync::new(
            participant_address,
            self.domain_participant.status_condition().address(),
            self.domain_participant
                .builtin_subscriber()
                .status_condition()
                .address(),
            self.domain_participant.domain_id(),
            self.domain_participant.instance_handle(),
            self.timer_driver.handle(),
        )
    }

    pub fn get_subscriber_async(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
        subscriber_handle: InstanceHandle,
    ) -> DdsResult<SubscriberAsync> {
        Ok(SubscriberAsync::new(
            subscriber_handle,
            self.domain_participant
                .get_subscriber(subscriber_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .status_condition()
                .address(),
            self.get_participant_async(participant_address),
        ))
    }

    pub fn get_data_reader_async<Foo>(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
        subscriber_handle: InstanceHandle,
        data_reader_handle: InstanceHandle,
    ) -> DdsResult<DataReaderAsync<Foo>> {
        let data_reader = self
            .domain_participant
            .get_subscriber(subscriber_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_reader(data_reader_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(DataReaderAsync::new(
            data_reader_handle,
            data_reader.status_condition().address(),
            self.get_subscriber_async(participant_address.clone(), subscriber_handle)?,
            self.get_topic_async(participant_address, data_reader.topic_name().to_owned())?,
        ))
    }

    pub fn get_publisher_async(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
        publisher_handle: InstanceHandle,
    ) -> DdsResult<PublisherAsync> {
        Ok(PublisherAsync::new(
            publisher_handle,
            self.domain_participant
                .get_publisher(publisher_handle)
                .ok_or(DdsError::AlreadyDeleted)?
                .status_condition()
                .address(),
            self.get_participant_async(participant_address),
        ))
    }

    pub fn get_data_writer_async<Foo>(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
        publisher_handle: InstanceHandle,
        data_writer_handle: InstanceHandle,
    ) -> DdsResult<DataWriterAsync<Foo>> {
        let data_writer = self
            .domain_participant
            .get_publisher(publisher_handle)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_data_writer(data_writer_handle)
            .ok_or(DdsError::AlreadyDeleted)?;

        Ok(DataWriterAsync::new(
            data_writer_handle,
            data_writer.status_condition().address(),
            self.get_publisher_async(participant_address.clone(), publisher_handle)?,
            self.get_topic_async(participant_address, data_writer.topic_name().to_owned())?,
        ))
    }

    pub fn get_topic_async(
        &self,
        participant_address: ActorAddress<DomainParticipantActor>,
        topic_name: String,
    ) -> DdsResult<TopicAsync> {
        let topic = self
            .domain_participant
            .get_topic(&topic_name)
            .ok_or(DdsError::AlreadyDeleted)?;
        Ok(TopicAsync::new(
            topic.instance_handle(),
            topic.status_condition().address(),
            topic.type_name().to_owned(),
            topic_name,
            self.get_participant_async(participant_address),
        ))
    }
}
