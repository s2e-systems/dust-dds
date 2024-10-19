use super::{
    data_reader_actor::DataReaderActor, data_writer_actor::DataWriterActor,
    domain_participant_actor,
};
use crate::{
    builtin_topics::{DCPS_PARTICIPANT, DCPS_PUBLICATION, DCPS_SUBSCRIPTION, DCPS_TOPIC},
    configuration::DustDdsConfiguration,
    dds_async::domain_participant_listener::DomainParticipantListenerAsync,
    domain::domain_participant_factory::DomainId,
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        actors::domain_participant_actor::DomainParticipantActor,
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_topic_data::DiscoveredTopicData,
            discovered_writer_data::DiscoveredWriterData,
        },
        data_representation_inline_qos::{
            parameter_id_values::PID_STATUS_INFO,
            types::{StatusInfo, STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED},
        },
        runtime::{executor::Executor, timer::TimerDriver},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{
            DataReaderQos, DataWriterQos, DomainParticipantFactoryQos, DomainParticipantQos,
            QosKind,
        },
        qos_policy::{
            DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
            ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::StatusKind,
        time::{Duration, DurationKind, DURATION_ZERO_NSEC, DURATION_ZERO_SEC},
    },
    rtps::{
        discovery_types::{
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        },
        participant::RtpsParticipant,
        reader::{ReaderCacheChange, ReaderHistoryCache},
        types::{Guid, GuidPrefix, ENTITYID_PARTICIPANT},
    },
    topic_definition::type_support::{DdsDeserialize, TypeSupport},
    xtypes::{deserialize::XTypesDeserialize, xcdr_deserializer::Xcdr1LeDeserializer},
};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, Mutex, OnceLock,
    },
};
use tracing::{error, warn};

#[derive(Default)]
pub struct DomainParticipantFactoryActor {
    domain_participant_list: HashMap<InstanceHandle, Actor<DomainParticipantActor>>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
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
    type Result = DdsResult<ActorAddress<DomainParticipantActor>>;
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

        let rtps_participant = RtpsParticipant::new(
            guid_prefix,
            message.domain_id,
            self.configuration.domain_tag().to_string(),
            self.configuration.interface_name(),
            self.configuration.udp_receive_buffer_size(),
        )?;
        let participant_guid = rtps_participant.guid();
        let transport = Arc::new(Mutex::new(rtps_participant));

        let domain_participant = DomainParticipantActor::new(
            Guid::new(guid_prefix, ENTITYID_PARTICIPANT),
            message.domain_id,
            self.configuration.domain_tag().to_string(),
            domain_participant_qos,
            message.listener,
            message.status_kind,
            executor,
            timer_driver,
            transport.clone(),
        );

        let participant_actor = Actor::spawn(domain_participant, &executor_handle);
        let participant_address = participant_actor.address();

        participant_actor.send_actor_mail(
            domain_participant_actor::CreateBuiltinParticipantsDetector {
                transport_reader: transport.lock().unwrap().create_builtin_stateless_reader(
                    Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER),
                    Box::new(SpdpBuiltinReaderHistoryCache {
                        participant_address: participant_address.clone(),
                    }),
                ),
            },
        );

        participant_actor.send_actor_mail(domain_participant_actor::CreateBuiltinTopicsDetector {
            transport_reader: transport.lock().unwrap().create_builtin_stateful_reader(
                Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR),
                Box::new(SedpBuiltinTopicsReaderHistoryCache {
                    participant_address: participant_address.clone(),
                }),
            ),
        });

        participant_actor.send_actor_mail(
            domain_participant_actor::CreateBuiltinPublicationsDetector {
                transport_reader: transport.lock().unwrap().create_builtin_stateful_reader(
                    Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR),
                    Box::new(SedpBuiltinPublicationsReaderHistoryCache {
                        participant_address: participant_address.clone(),
                    }),
                ),
            },
        );

        participant_actor.send_actor_mail(
            domain_participant_actor::CreateBuiltinSubscriptionsDetector {
                transport_reader: transport.lock().unwrap().create_builtin_stateful_reader(
                    Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR),
                    Box::new(SedpBuiltinSubscriptionsReaderHistoryCache {
                        participant_address: participant_address,
                    }),
                ),
            },
        );

        participant_actor.send_actor_mail(
            domain_participant_actor::CreateBuiltinParticipantsAnnouncer {
                transport_writer: transport.lock().unwrap().create_builtin_stateless_writer(
                    Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER),
                ),
            },
        );

        participant_actor.send_actor_mail(domain_participant_actor::CreateBuiltinTopicsAnnouncer {
            transport_writer: transport
                .lock()
                .unwrap()
                .create_builtin_stateful_writer(Guid::new(
                    guid_prefix,
                    ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
                )),
        });

        participant_actor.send_actor_mail(
            domain_participant_actor::CreateBuiltinPublicationsAnnouncer {
                transport_writer: transport.lock().unwrap().create_builtin_stateful_writer(
                    Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER),
                ),
            },
        );

        participant_actor.send_actor_mail(
            domain_participant_actor::CreateBuiltinSubscriptionsAnnouncer {
                transport_writer: transport.lock().unwrap().create_builtin_stateful_writer(
                    Guid::new(guid_prefix, ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER),
                ),
            },
        );

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

        // let participant_clone = participant.clone();
        // executor_handle.spawn(async move {
        //     loop {
        //         let r = participant_clone.announce_participant().await;
        //         if r.is_err() {
        //             break;
        //         }

        //         on_participant_discovery_receiver.recv().await;
        //     }
        // });

        let participant_address = participant_actor.address();
        self.domain_participant_list.insert(
            InstanceHandle::new(participant_guid.into()),
            participant_actor,
        );
        Ok(participant_address)
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

pub fn sedp_data_reader_qos() -> DataReaderQos {
    DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(
                DURATION_ZERO_SEC,
                DURATION_ZERO_NSEC,
            )),
        },
        ..Default::default()
    }
}

pub fn sedp_data_writer_qos() -> DataWriterQos {
    DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(1),
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Finite(Duration::new(
                DURATION_ZERO_SEC,
                DURATION_ZERO_NSEC,
            )),
        },
        ..Default::default()
    }
}

struct SpdpBuiltinReaderHistoryCache {
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for SpdpBuiltinReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        self.participant_address
            .send_actor_mail(domain_participant_actor::AddDiscoveredParticipant { cache_change });
    }
}

struct SedpBuiltinTopicsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for SedpBuiltinTopicsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        if let Ok(discovered_topic_data) =
            DiscoveredTopicData::deserialize_data(cache_change.data_value.as_ref())
        {
            todo!()
            // self.participant_address
            //     .send_actor_mail(domain_participant_actor::AddMatchedTopic {
            //         discovered_topic_data,
            //         // participant: participant.clone(),
            //     })
            //     .ok();
        }
        todo!()
        // self.subscriber_address
        //     .send_actor_mail(subscriber_actor::AddChange {
        //         cache_change,
        //         reader_instance_handle: self.reader_instance_handle,
        //     })
        //     .ok();
    }
}

struct SedpBuiltinPublicationsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for SedpBuiltinPublicationsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        if let Some(p) = cache_change
            .inline_qos
            .parameter()
            .iter()
            .find(|&x| x.parameter_id() == PID_STATUS_INFO)
        {
            let mut deserializer = Xcdr1LeDeserializer::new(p.value());
            let status_info: StatusInfo =
                XTypesDeserialize::deserialize(&mut deserializer).unwrap();
            if status_info == STATUS_INFO_DISPOSED
                || status_info == STATUS_INFO_DISPOSED_UNREGISTERED
            {
                if let Ok(discovered_writer_handle) =
                    InstanceHandle::deserialize_data(cache_change.data_value.as_ref())
                {
                    todo!()
                    // self.participant_address
                    //     .send_actor_mail(domain_participant_actor::RemoveMatchedWriter {
                    //         discovered_writer_handle,
                    //     })
                    //     .ok();
                }
            }
        } else {
            if let Ok(discovered_writer_data) =
                DiscoveredWriterData::deserialize_data(cache_change.data_value.as_ref())
            {
                todo!()
                // self.participant_address
                //     .send_actor_mail(domain_participant_actor::AddMatchedWriter {
                //         discovered_writer_data,
                //         // participant: participant.clone(),
                //     })
                //     .ok();
            }
        }
        todo!()
        // self.subscriber_address
        //     .send_actor_mail(subscriber_actor::AddChange {
        //         cache_change,
        //         reader_instance_handle: self.reader_instance_handle,
        //     })
        //     .ok();
    }
}

struct SedpBuiltinSubscriptionsReaderHistoryCache {
    pub participant_address: ActorAddress<DomainParticipantActor>,
}

impl ReaderHistoryCache for SedpBuiltinSubscriptionsReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange) {
        if let Some(p) = cache_change
            .inline_qos
            .parameter()
            .iter()
            .find(|&x| x.parameter_id() == PID_STATUS_INFO)
        {
            let mut deserializer = Xcdr1LeDeserializer::new(p.value());
            let status_info: StatusInfo =
                XTypesDeserialize::deserialize(&mut deserializer).unwrap();
            if status_info == STATUS_INFO_DISPOSED
                || status_info == STATUS_INFO_DISPOSED_UNREGISTERED
            {
                if let Ok(discovered_reader_handle) =
                    InstanceHandle::deserialize_data(cache_change.data_value.as_ref())
                {
                    todo!()
                    // self.participant_address
                    //     .send_actor_mail(domain_participant_actor::RemoveMatchedReader {
                    //         discovered_reader_handle,
                    //     })
                    //     .ok();
                }
            }
        } else {
            if let Ok(discovered_reader_data) =
                DiscoveredReaderData::deserialize_data(cache_change.data_value.as_ref())
            {
                todo!()
                // self.participant_address
                //     .send_actor_mail(domain_participant_actor::AddMatchedReader {
                //         discovered_reader_data,
                //         // participant: participant.clone(),
                //     })
                //     .ok();
            }
        }
        todo!()
        // self.subscriber_address
        //     .send_actor_mail(subscriber_actor::AddChange {
        //         cache_change,
        //         reader_instance_handle: self.reader_instance_handle,
        //     })
        //     .ok();
    }
}
