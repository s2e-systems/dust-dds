use super::{domain_participant_actor, handle::ParticipantHandle};
use crate::{
    configuration::DustDdsConfiguration,
    dds_async::domain_participant_listener::DomainParticipantListenerAsync,
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::{Actor, ActorAddress, ActorBuilder, Mail, MailHandler},
        actors::domain_participant_actor::DomainParticipantActor,
        runtime::{executor::Executor, timer::TimerDriver},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    rtps::{
        reader::{ReaderCacheChange, ReaderHistoryCache},
        transport::RtpsTransport,
        types::GuidPrefix,
    },
};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{
        atomic::{AtomicU32, Ordering},
        OnceLock,
    },
};
use tracing::{error, warn};

#[derive(Default)]
pub struct DomainParticipantFactoryActor {
    domain_participant_list: HashMap<InstanceHandle, Actor<DomainParticipantActor>>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
    participant_counter: u8,
}

impl DomainParticipantFactoryActor {
    pub fn new() -> Self {
        Default::default()
    }

    fn get_unique_participant_id(&mut self) -> u32 {
        static COUNTER: OnceLock<AtomicU32> = OnceLock::new();
        let c = COUNTER.get_or_init(|| AtomicU32::new(0));
        c.fetch_add(1, Ordering::Acquire)
    }

    fn create_new_guid_prefix(&mut self) -> GuidPrefix {
        let interface_address = NetworkInterface::show()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter(|x| {
                if let Some(if_name) = self.configuration.interface_name() {
                    &x.name == if_name
                } else {
                    true
                }
            })
            .flat_map(|i| {
                i.addr
                    .into_iter()
                    .filter(|a| matches!(a, Addr::V4(v4) if !v4.ip.is_loopback()))
            })
            .next();
        let host_id = if let Some(interface) = interface_address {
            match interface.ip() {
                IpAddr::V4(a) => a.octets(),
                IpAddr::V6(_) => unimplemented!("IPv6 not yet implemented"),
            }
        } else {
            warn!("Failed to get Host ID from IP address, use 0 instead");
            [0; 4]
        };

        let app_id = std::process::id().to_ne_bytes();
        let instance_id = self.get_unique_participant_id().to_ne_bytes();

        [
            host_id[0],
            host_id[1],
            host_id[2],
            host_id[3], // Host ID
            app_id[0],
            app_id[1],
            app_id[2],
            app_id[3], // App ID
            instance_id[0],
            instance_id[1],
            instance_id[2],
            instance_id[3], // Instance ID
        ]
    }
}

pub struct CreateParticipant {
    pub domain_id: DomainId,
    pub qos: QosKind<DomainParticipantQos>,
    pub listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for CreateParticipant {
    type Result = DdsResult<(ActorAddress<DomainParticipantActor>, InstanceHandle)>;
}
impl MailHandler<CreateParticipant> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: CreateParticipant) -> <CreateParticipant as Mail>::Result {
        let executor = Executor::new();
        let executor_handle = executor.handle();

        let timer_driver = TimerDriver::new();
        let timer_handle = timer_driver.handle();

        let domain_participant_qos = match message.qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let guid_prefix = self.create_new_guid_prefix();
        let participant_actor_builder = ActorBuilder::new();

        let transport = Box::new(RtpsTransport::new(
            guid_prefix,
            message.domain_id,
            self.configuration.domain_tag().to_string(),
            self.configuration.interface_name(),
            self.configuration.udp_receive_buffer_size(),
            Box::new(DcpsParticipantReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
            Box::new(DcpsTopicsReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
            Box::new(DcpsPublicationsReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
            Box::new(DcpsSubscriptionsReaderHistoryCache {
                participant_address: participant_actor_builder.address(),
            }),
        )?);
        let participant_handle = ParticipantHandle::new(self.participant_counter);

        let domain_participant = DomainParticipantActor::new(
            participant_handle,
            message.domain_id,
            self.configuration.domain_tag().to_string(),
            domain_participant_qos,
            message.listener,
            message.status_kind,
            executor,
            timer_driver,
            transport,
        );

        let participant_actor =
            participant_actor_builder.build(domain_participant, &executor_handle);

        //****** Spawn the participant actor and tasks **********//

        // Start the regular participant announcement task
        let participant_address = participant_actor.address();
        let participant_announcement_interval =
            self.configuration.participant_announcement_interval();

        executor_handle.spawn(async move {
            loop {
                if let Ok(r) = participant_address
                    .send_actor_mail(domain_participant_actor::AnnounceParticipant)
                {
                    if let Err(announce_result) = r.receive_reply().await {
                        error!("Error announcing participant: {:?}", announce_result);
                    }
                    timer_handle.sleep(participant_announcement_interval).await;
                } else {
                    break;
                }
            }
        });

        let participant_address = participant_actor.address();
        self.domain_participant_list
            .insert(participant_handle.into(), participant_actor);

        Ok((participant_address, participant_handle.into()))
    }
}

pub struct DeleteParticipant {
    pub handle: InstanceHandle,
}
impl Mail for DeleteParticipant {
    type Result = DdsResult<Actor<DomainParticipantActor>>;
}
impl MailHandler<DeleteParticipant> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: DeleteParticipant) -> <DeleteParticipant as Mail>::Result {
        self.domain_participant_list
            .remove(&message.handle)
            .ok_or(DdsError::PreconditionNotMet(
                "Participant can only be deleted from its parent domain participant factory"
                    .to_string(),
            ))
    }
}

pub struct GetParticipantList;
impl Mail for GetParticipantList {
    type Result = Vec<ActorAddress<DomainParticipantActor>>;
}
impl MailHandler<GetParticipantList> for DomainParticipantFactoryActor {
    fn handle(&mut self, _: GetParticipantList) -> <GetParticipantList as Mail>::Result {
        self.domain_participant_list
            .values()
            .map(|a| a.address())
            .collect()
    }
}

pub struct SetDefaultParticipantQos {
    pub qos: QosKind<DomainParticipantQos>,
}
impl Mail for SetDefaultParticipantQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultParticipantQos> for DomainParticipantFactoryActor {
    fn handle(
        &mut self,
        message: SetDefaultParticipantQos,
    ) -> <SetDefaultParticipantQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_participant_qos = qos;

        Ok(())
    }
}

pub struct GetDefaultParticipantQos;
impl Mail for GetDefaultParticipantQos {
    type Result = DomainParticipantQos;
}
impl MailHandler<GetDefaultParticipantQos> for DomainParticipantFactoryActor {
    fn handle(
        &mut self,
        _: GetDefaultParticipantQos,
    ) -> <GetDefaultParticipantQos as Mail>::Result {
        self.default_participant_qos.clone()
    }
}

pub struct SetQos {
    pub qos: QosKind<DomainParticipantFactoryQos>,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;

        Ok(())
    }
}

pub struct GetQos;
impl Mail for GetQos {
    type Result = DomainParticipantFactoryQos;
}
impl MailHandler<GetQos> for DomainParticipantFactoryActor {
    fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct SetConfiguration {
    pub configuration: DustDdsConfiguration,
}
impl Mail for SetConfiguration {
    type Result = ();
}
impl MailHandler<SetConfiguration> for DomainParticipantFactoryActor {
    fn handle(&mut self, message: SetConfiguration) -> <SetConfiguration as Mail>::Result {
        self.configuration = message.configuration;
    }
}

pub struct GetConfiguration;
impl Mail for GetConfiguration {
    type Result = DustDdsConfiguration;
}
impl MailHandler<GetConfiguration> for DomainParticipantFactoryActor {
    fn handle(&mut self, _: GetConfiguration) -> <GetConfiguration as Mail>::Result {
        self.configuration.clone()
    }
}

struct DcpsParticipantReaderHistoryCache {
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for DcpsParticipantReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        self.participant_address
            .send_actor_mail(
                domain_participant_actor::AddBuiltinParticipantsDetectorCacheChange {
                    cache_change,
                },
            )
            .ok();
    }
}

struct DcpsTopicsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for DcpsTopicsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        self.participant_address
            .send_actor_mail(
                domain_participant_actor::AddBuiltinTopicsDetectorCacheChange { cache_change },
            )
            .ok();
    }
}

struct DcpsSubscriptionsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for DcpsSubscriptionsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        self.participant_address
            .send_actor_mail(
                domain_participant_actor::AddBuiltinSubscriptionsDetectorCacheChange {
                    cache_change,
                },
            )
            .ok();
    }
}

struct DcpsPublicationsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for DcpsPublicationsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        self.participant_address
            .send_actor_mail(
                domain_participant_actor::AddBuiltinPublicationsDetectorCacheChange {
                    cache_change,
                },
            )
            .ok();
    }
}
