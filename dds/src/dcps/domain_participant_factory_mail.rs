use crate::{
    dcps::{
        actor::ActorAddress,
        channels::{mpsc::MpscSender, oneshot::OneshotSender},
        domain_participant_factory::DcpsParticipantFactory,
        domain_participant_mail::DcpsDomainParticipantMail,
        listeners::domain_participant_listener::DcpsDomainParticipantListener,
        status_condition::DcpsStatusCondition,
    },
    dds_async::configuration::DustDdsConfiguration,
    infrastructure::{
        domain::DomainId,
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    runtime::DdsRuntime,
    transport::interface::TransportParticipantFactory,
};
use alloc::vec::Vec;

pub enum DcpsParticipantFactoryMail<R: DdsRuntime> {
    CreateParticipant {
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        status_kind: Vec<StatusKind>,
        clock_handle: R::ClockHandle,
        timer_handle: R::TimerHandle,
        spawner_handle: R::SpawnerHandle,
        #[allow(clippy::type_complexity)]
        reply_sender: OneshotSender<
            DdsResult<(
                MpscSender<DcpsDomainParticipantMail>,
                InstanceHandle,
                ActorAddress<DcpsStatusCondition>,
            )>,
        >,
    },
    DeleteParticipant {
        handle: InstanceHandle,
        reply_sender: OneshotSender<DdsResult<MpscSender<DcpsDomainParticipantMail>>>,
    },
    SetDefaultParticipantQos {
        qos: QosKind<DomainParticipantQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetDefaultParticipantQos {
        reply_sender: OneshotSender<DomainParticipantQos>,
    },
    SetQos {
        qos: QosKind<DomainParticipantFactoryQos>,
        reply_sender: OneshotSender<DdsResult<()>>,
    },
    GetQos {
        reply_sender: OneshotSender<DomainParticipantFactoryQos>,
    },
    SetConfiguration {
        configuration: DustDdsConfiguration,
    },
    GetConfiguration {
        reply_sender: OneshotSender<DustDdsConfiguration>,
    },
}

impl<T: TransportParticipantFactory> DcpsParticipantFactory<T> {
    pub async fn handle<R: DdsRuntime>(&mut self, message: DcpsParticipantFactoryMail<R>) {
        match message {
            DcpsParticipantFactoryMail::CreateParticipant {
                domain_id,
                qos,
                dcps_listener,
                status_kind,
                clock_handle,
                timer_handle,
                spawner_handle,
                reply_sender,
            } => reply_sender.send(
                self.create_participant::<R>(
                    domain_id,
                    qos,
                    dcps_listener,
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
