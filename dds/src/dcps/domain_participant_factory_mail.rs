use crate::{
    configuration::DustDdsConfiguration,
    dcps::{
        actor::{ActorAddress, MailHandler},
        domain_participant_factory::DomainParticipantFactoryActor,
        domain_participant_mail::DomainParticipantMail,
        listeners::domain_participant_listener::ListenerMail,
        status_condition_actor::StatusConditionActor,
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

pub enum DomainParticipantFactoryMail<R: DdsRuntime> {
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
                R::ChannelSender<DomainParticipantMail<R>>,
                InstanceHandle,
                ActorAddress<R, StatusConditionActor<R>>,
            )>,
        >,
    },
    DeleteParticipant {
        handle: InstanceHandle,
        reply_sender: R::OneshotSender<DdsResult<R::ChannelSender<DomainParticipantMail<R>>>>,
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

impl<R: DdsRuntime, T: TransportParticipantFactory> MailHandler
    for DomainParticipantFactoryActor<R, T>
{
    type Mail = DomainParticipantFactoryMail<R>;

    async fn handle(&mut self, message: Self::Mail) {
        match message {
            DomainParticipantFactoryMail::CreateParticipant {
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
            DomainParticipantFactoryMail::DeleteParticipant {
                handle,
                reply_sender,
            } => reply_sender.send(self.delete_participant(handle)),
            DomainParticipantFactoryMail::SetDefaultParticipantQos { qos, reply_sender } => {
                reply_sender.send(self.set_default_participant_qos(qos))
            }
            DomainParticipantFactoryMail::GetDefaultParticipantQos { reply_sender } => {
                reply_sender.send(self.get_default_participant_qos())
            }
            DomainParticipantFactoryMail::SetQos { qos, reply_sender } => {
                reply_sender.send(self.set_qos(qos))
            }
            DomainParticipantFactoryMail::GetQos { reply_sender } => {
                reply_sender.send(self.get_qos())
            }
            DomainParticipantFactoryMail::SetConfiguration { configuration } => {
                self.set_configuration(configuration)
            }
            DomainParticipantFactoryMail::GetConfiguration { reply_sender } => {
                reply_sender.send(self.get_configuration())
            }
        }
    }
}
