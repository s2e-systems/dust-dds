use crate::{
    dds_async::publisher_listener::PublisherListenerAsync,
    implementation::{
        any_data_writer_listener::AnyDataWriterListener,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor,
            entities::data_writer::{DataWriterEntity, TransportWriterKind},
        },
        listeners::{
            data_writer_listener::DataWriterListenerActor,
            publisher_listener::PublisherListenerActor,
        },
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        qos_policy::ReliabilityQosPolicyKind,
        status::StatusKind,
    },
    runtime::{
        actor::{Actor, ActorAddress, MailHandler},
        oneshot::OneshotSender,
    },
    transport::types::{
        EntityId, ReliabilityKind, TopicKind, USER_DEFINED_WRITER_NO_KEY,
        USER_DEFINED_WRITER_WITH_KEY,
    },
    xtypes::dynamic_type::DynamicType,
};

use super::{data_writer_service, discovery_service};

pub struct CreateDataWriter {
    pub publisher_handle: InstanceHandle,
    pub topic_name: String,
    pub qos: QosKind<DataWriterQos>,
    pub a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub mask: Vec<StatusKind>,
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub reply_sender:
        OneshotSender<DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)>>,
}
impl MailHandler<CreateDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: CreateDataWriter) {
        let Some(topic) = self.domain_participant.get_topic(&message.topic_name) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        let topic_kind = get_topic_kind(topic.type_support().as_ref());
        let type_support = topic.type_support().clone();
        let type_name = topic.type_name().to_owned();
        let entity_kind = match topic_kind {
            TopicKind::WithKey => USER_DEFINED_WRITER_WITH_KEY,
            TopicKind::NoKey => USER_DEFINED_WRITER_NO_KEY,
        };

        self.entity_counter += 1;
        let entity_id = EntityId::new(
            [
                0,
                self.entity_counter.to_le_bytes()[0],
                self.entity_counter.to_le_bytes()[1],
            ],
            entity_kind,
        );

        let writer_handle = self.instance_handle_counter.generate_new_instance_handle();
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        let qos = match message.qos {
            QosKind::Default => publisher.default_datawriter_qos().clone(),
            QosKind::Specific(q) => {
                if q.is_consistent().is_ok() {
                    q
                } else {
                    message.reply_sender.send(Err(DdsError::InconsistentPolicy));
                    return;
                }
            }
        };
        let reliablity_kind = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
        };
        let transport_writer = self
            .transport
            .create_stateful_writer(entity_id, reliablity_kind);

        let topic_name = message.topic_name;

        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            &self.listener_executor.handle(),
        );
        let writer_status_condition_address = status_condition.address();
        let listener = message.a_listener.map(|l| {
            Actor::spawn(
                DataWriterListenerActor::new(l),
                &self.listener_executor.handle(),
            )
        });
        let data_writer = DataWriterEntity::new(
            writer_handle,
            TransportWriterKind::Stateful(transport_writer),
            topic_name,
            type_name,
            type_support,
            status_condition,
            listener,
            message.mask,
            qos,
        );
        let data_writer_handle = data_writer.instance_handle();

        publisher.insert_data_writer(data_writer);

        if publisher.enabled() && publisher.qos().entity_factory.autoenable_created_entities {
            message
                .participant_address
                .send_actor_mail(data_writer_service::Enable {
                    publisher_handle: message.publisher_handle,
                    data_writer_handle: writer_handle,
                    participant_address: message.participant_address.clone(),
                })
                .ok();
        }

        message
            .reply_sender
            .send(Ok((data_writer_handle, writer_status_condition_address)));
    }
}

pub struct DeleteDataWriter {
    pub publisher_handle: InstanceHandle,
    pub datawriter_handle: InstanceHandle,
    pub participant_address: ActorAddress<DomainParticipantActor>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<DeleteDataWriter> for DomainParticipantActor {
    fn handle(&mut self, message: DeleteDataWriter) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        let Some(data_writer) = publisher.remove_data_writer(message.datawriter_handle) else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message
            .participant_address
            .send_actor_mail(discovery_service::AnnounceDeletedDataWriter { data_writer })
            .ok();
        message.reply_sender.send(Ok(()))
    }
}

pub struct SetDefaultDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub qos: QosKind<DataWriterQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetDefaultDataWriterQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetDefaultDataWriterQos) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        let qos = match message.qos {
            QosKind::Default => DataWriterQos::default(),
            QosKind::Specific(q) => q,
        };
        publisher.set_default_datawriter_qos(qos);
        message.reply_sender.send(Ok(()));
    }
}

pub struct GetDefaultDataWriterQos {
    pub publisher_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<DataWriterQos>>,
}

impl MailHandler<GetDefaultDataWriterQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetDefaultDataWriterQos) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        message
            .reply_sender
            .send(Ok(publisher.default_datawriter_qos().clone()));
    }
}

pub struct SetQos {
    pub publisher_handle: InstanceHandle,
    pub qos: QosKind<PublisherQos>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetQos> for DomainParticipantActor {
    fn handle(&mut self, message: SetQos) {
        let qos = match message.qos {
            QosKind::Default => self.domain_participant.default_publisher_qos().clone(),
            QosKind::Specific(q) => q,
        };
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(publisher.set_qos(qos));
    }
}

pub struct GetQos {
    pub publisher_handle: InstanceHandle,
    pub reply_sender: OneshotSender<DdsResult<PublisherQos>>,
}
impl MailHandler<GetQos> for DomainParticipantActor {
    fn handle(&mut self, message: GetQos) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };

        message.reply_sender.send(Ok(publisher.qos().clone()));
    }
}

pub struct SetListener {
    pub publisher_handle: InstanceHandle,
    pub a_listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub mask: Vec<StatusKind>,
    pub reply_sender: OneshotSender<DdsResult<()>>,
}
impl MailHandler<SetListener> for DomainParticipantActor {
    fn handle(&mut self, message: SetListener) {
        let Some(publisher) = self
            .domain_participant
            .get_mut_publisher(message.publisher_handle)
        else {
            message.reply_sender.send(Err(DdsError::AlreadyDeleted));
            return;
        };
        publisher.set_listener(
            message.a_listener.map(|l| {
                Actor::spawn(
                    PublisherListenerActor::new(l),
                    &self.listener_executor.handle(),
                )
            }),
            message.mask,
        );

        message.reply_sender.send(Ok(()));
    }
}

fn get_topic_kind(type_support: &dyn DynamicType) -> TopicKind {
    for index in 0..type_support.get_member_count() {
        if let Ok(m) = type_support.get_member_by_index(index) {
            if let Ok(d) = m.get_descriptor() {
                if d.is_key {
                    return TopicKind::WithKey;
                }
            }
        }
    }
    TopicKind::NoKey
}
