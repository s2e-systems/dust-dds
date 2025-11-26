use super::actor::MailHandler;
use crate::{
    dcps::{
        actor::ActorAddress,
        channels::{
            mpsc::{MpscSender, mpsc_channel},
            oneshot::oneshot,
        },
        domain_participant::DcpsDomainParticipant,
        domain_participant_mail::{
            DcpsDomainParticipantMail, DiscoveryServiceMail, MessageServiceMail,
            ParticipantServiceMail,
        },
        listeners::domain_participant_listener::DcpsDomainParticipantListener,
        status_condition::DcpsStatusCondition,
    },
    dds_async::configuration::DustDdsConfiguration,
    infrastructure::{
        domain::DomainId,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    runtime::{DdsRuntime, Spawner, Timer},
    transport::{interface::TransportParticipantFactory, types::GuidPrefix},
};
use alloc::{string::ToString, sync::Arc, vec::Vec};
use core::{future::Future, marker::PhantomData, pin::Pin, task::Poll};

pub struct DcpsParticipantFactory<R: DdsRuntime, T> {
    domain_participant_list: Vec<(InstanceHandle, MpscSender<DcpsDomainParticipantMail>)>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
    transport: T,
    entity_counter: u32,
    app_id: [u8; 4],
    host_id: [u8; 4],
    runtime: PhantomData<R>,
}

impl<R: DdsRuntime, T: TransportParticipantFactory> DcpsParticipantFactory<R, T> {
    pub fn new(app_id: [u8; 4], host_id: [u8; 4], transport: T) -> Self {
        Self {
            domain_participant_list: Default::default(),
            qos: Default::default(),
            default_participant_qos: Default::default(),
            configuration: Default::default(),
            transport,
            entity_counter: 0,
            app_id,
            host_id,
            runtime: PhantomData,
        }
    }

    fn get_unique_participant_id(&mut self) -> u32 {
        let id = self.entity_counter;
        self.entity_counter += 1;
        id
    }

    fn create_new_guid_prefix(&mut self) -> GuidPrefix {
        let instance_id = self.get_unique_participant_id().to_ne_bytes();

        [
            self.host_id[0],
            self.host_id[1],
            self.host_id[2],
            self.host_id[3], // Host ID
            self.app_id[0],
            self.app_id[1],
            self.app_id[2],
            self.app_id[3], // App ID
            instance_id[0],
            instance_id[1],
            instance_id[2],
            instance_id[3], // Instance ID
        ]
    }

    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    pub async fn create_participant(
        &mut self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        status_kind: Vec<StatusKind>,
        clock_handle: R::ClockHandle,
        mut timer_handle: R::TimerHandle,
        spawner_handle: R::SpawnerHandle,
    ) -> DdsResult<(
        MpscSender<DcpsDomainParticipantMail>,
        InstanceHandle,
        ActorAddress<DcpsStatusCondition>,
    )> {
        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };
        let (participant_sender, participant_receiver) = mpsc_channel();
        let (data_channel_sender, data_channel_receiver) = mpsc_channel();

        let guid_prefix = self.create_new_guid_prefix();
        let transport = self
            .transport
            .create_participant(domain_id, data_channel_sender)
            .await;

        let listener_sender = dcps_listener.map(|l| l.spawn::<R>(&spawner_handle));

        let mut dcps_participant: DcpsDomainParticipant<R> = DcpsDomainParticipant::new(
            domain_id,
            String::from(self.configuration.domain_tag()),
            guid_prefix,
            domain_participant_qos,
            listener_sender,
            status_kind,
            transport,
            participant_sender.clone(),
            clock_handle,
            timer_handle.clone(),
            spawner_handle.clone(),
        );
        let participant_instance_handle = dcps_participant.get_instance_handle();
        let builtin_subscriber_status_condition_address = dcps_participant
            .get_builtin_subscriber_status_condition()
            .address();

        enum Either<A, B> {
            A(A),
            B(B),
        }
        struct Select<A, B> {
            a: A,
            b: B,
        }
        impl<A, B> Future for Select<A, B>
        where
            A: Future<Output = Option<DcpsDomainParticipantMail>> + Unpin,
            B: Future<Output = Option<Arc<[u8]>>> + Unpin,
        {
            type Output = Either<Option<DcpsDomainParticipantMail>, Option<Arc<[u8]>>>;

            fn poll(
                mut self: Pin<&mut Self>,
                cx: &mut core::task::Context<'_>,
            ) -> core::task::Poll<Self::Output> {
                if let Poll::Ready(a) = Pin::new(&mut self.a).poll(cx) {
                    return Poll::Ready(Either::A(a));
                }
                if let Poll::Ready(b) = Pin::new(&mut self.b).poll(cx) {
                    return Poll::Ready(Either::B(b));
                }
                Poll::Pending
            }
        }

        spawner_handle.spawn(async move {
            loop {
                let select = Select {
                    a: core::pin::pin!(participant_receiver.receive()),
                    b: core::pin::pin!(data_channel_receiver.receive()),
                };
                match select.await {
                    Either::A(dcps_domain_participant_mail) => {
                        if let Some(dcps_domain_participant_mail) = dcps_domain_participant_mail {
                            dcps_participant.handle(dcps_domain_participant_mail).await
                        }
                    }
                    Either::B(data_message) => {
                        if let Some(data_message) = data_message {
                            dcps_participant.handle_data(data_message).await
                        }
                    }
                }
            }
        });

        //****** Spawn the participant actor and tasks **********//

        // Start the regular participant announcement task
        let participant_address = participant_sender.clone();
        let mut timer_handle_clone = timer_handle.clone();
        let participant_announcement_interval =
            self.configuration.participant_announcement_interval();

        spawner_handle.spawn(async move {
            while participant_address
                .send(DcpsDomainParticipantMail::Discovery(
                    DiscoveryServiceMail::AnnounceParticipant,
                ))
                .await
                .is_ok()
            {
                timer_handle_clone
                    .delay(participant_announcement_interval)
                    .await;
            }
        });

        // Start regular message writing
        let participant_address = participant_sender.clone();
        spawner_handle.spawn(async move {
            while participant_address
                .send(DcpsDomainParticipantMail::Message(MessageServiceMail::Poke))
                .await
                .is_ok()
            {
                timer_handle
                    .delay(core::time::Duration::from_millis(50))
                    .await;
            }
        });

        if self.qos.entity_factory.autoenable_created_entities {
            let (reply_sender, _reply_receiver) = oneshot();
            participant_sender
                .send(DcpsDomainParticipantMail::Participant(
                    ParticipantServiceMail::Enable { reply_sender },
                ))
                .await
                .ok();
        }

        let participant_address = participant_sender.clone();
        self.domain_participant_list
            .push((participant_instance_handle, participant_sender));

        Ok((
            participant_address,
            participant_instance_handle,
            builtin_subscriber_status_condition_address,
        ))
    }

    pub fn delete_participant(
        &mut self,
        handle: InstanceHandle,
    ) -> DdsResult<MpscSender<DcpsDomainParticipantMail>> {
        let index = self
            .domain_participant_list
            .iter()
            .position(|(h, _)| h == &handle)
            .ok_or(DdsError::PreconditionNotMet(
                "Participant can only be deleted from its parent domain participant factory"
                    .to_string(),
            ))?;

        let (_, participant) = self.domain_participant_list.remove(index);
        Ok(participant)
    }

    pub fn set_default_participant_qos(
        &mut self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_participant_qos = qos;

        Ok(())
    }

    pub fn get_default_participant_qos(&mut self) -> DomainParticipantQos {
        self.default_participant_qos.clone()
    }

    pub fn set_qos(&mut self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&mut self) -> DomainParticipantFactoryQos {
        self.qos.clone()
    }

    pub fn set_configuration(&mut self, configuration: DustDdsConfiguration) {
        self.configuration = configuration;
    }

    pub fn get_configuration(&mut self) -> DustDdsConfiguration {
        self.configuration.clone()
    }
}
