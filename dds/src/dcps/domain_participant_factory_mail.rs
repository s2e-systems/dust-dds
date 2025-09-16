use crate::{
    configuration::DustDdsConfiguration,
    dcps::{
        actor::{ActorAddress, MailHandler},
        domain_participant_factory::DcpsParticipantFactory,
        domain_participant_mail::DcpsDomainParticipantMail,
        listeners::domain_participant_listener::ListenerMail,
        status_condition::DcpsStatusCondition,
    },
    infrastructure::{
        domain::DomainId,
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    runtime::{DdsRuntime, OneshotSend},
    transport::interface::TransportParticipantFactory,
};

pub enum DcpsParticipantFactoryMail<R: DdsRuntime> {
    CreateParticipant {
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        listener_sender: Option<R::ChannelSender<ListenerMail<R>>>,
        status_kind: Vec<StatusKind>,
        clock_handle: R::ClockHandle,
        timer_handle: R::TimerHandle,
        spawner_handle: R::SpawnerHandle,
        #[allow(clippy::type_complexity)]
        reply_sender: R::OneshotSender<
            DdsResult<(
                R::ChannelSender<DcpsDomainParticipantMail<R>>,
                InstanceHandle,
                ActorAddress<R, DcpsStatusCondition<R>>,
            )>,
        >,
    },
    DeleteParticipant {
        handle: InstanceHandle,
        reply_sender: R::OneshotSender<DdsResult<R::ChannelSender<DcpsDomainParticipantMail<R>>>>,
    },
    SetDefaultParticipantQos {
        qos: QosKind<DomainParticipantQos>,
        reply_sender: R::OneshotSender<DdsResult<()>>,
    },
    GetDefaultParticipantQos {
        reply_sender: R::OneshotSender<DomainParticipantQos>,
    },
    SetQos {
        qos: QosKind<DomainParticipantFactoryQos>,
        reply_sender: R::OneshotSender<DdsResult<()>>,
    },
    GetQos {
        reply_sender: R::OneshotSender<DomainParticipantFactoryQos>,
    },
    SetConfiguration {
        configuration: DustDdsConfiguration,
    },
    GetConfiguration {
        reply_sender: R::OneshotSender<DustDdsConfiguration>,
    },
}

impl<R: DdsRuntime, T: TransportParticipantFactory> MailHandler for DcpsParticipantFactory<R, T> {
    type Mail = DcpsParticipantFactoryMail<R>;

    async fn handle(&mut self, message: Self::Mail) {
        match message {
            DcpsParticipantFactoryMail::CreateParticipant {
                domain_id,
                qos,
                listener_sender,
                status_kind,
                clock_handle,
                timer_handle,
                spawner_handle,
                reply_sender,
            } => reply_sender.send(
                self.create_participant(
                    domain_id,
                    qos,
                    listener_sender,
                    status_kind,
                    clock_handle,
                    timer_handle,
                    spawner_handle,
                )
                .await,
            ),
            DcpsParticipantFactoryMail::DeleteParticipant {
                handle,
                reply_sender,
            } => reply_sender.send(self.delete_participant(handle)),
            DcpsParticipantFactoryMail::SetDefaultParticipantQos { qos, reply_sender } => {
                reply_sender.send(self.set_default_participant_qos(qos))
            }
            DcpsParticipantFactoryMail::GetDefaultParticipantQos { reply_sender } => {
                reply_sender.send(self.get_default_participant_qos())
            }
            DcpsParticipantFactoryMail::SetQos { qos, reply_sender } => {
                reply_sender.send(self.set_qos(qos))
            }
            DcpsParticipantFactoryMail::GetQos { reply_sender } => {
                reply_sender.send(self.get_qos())
            }
            DcpsParticipantFactoryMail::SetConfiguration { configuration } => {
                self.set_configuration(configuration)
            }
            DcpsParticipantFactoryMail::GetConfiguration { reply_sender } => {
                reply_sender.send(self.get_configuration())
            }
        }
    }
}
