use std::sync::{
    atomic::{self, AtomicBool},
    mpsc::{Receiver, SyncSender},
    Arc,
};

use async_std::prelude::StreamExt;
use dds_api::{
    builtin_topics::ParticipantBuiltinTopicData,
    dcps_psm::{BuiltInTopicKey, Time},
    domain::domain_participant::DomainParticipant,
    publication::{data_writer::DataWriter, publisher::Publisher},
    return_type::{DDSError, DDSResult},
    subscription::{data_reader::DataReader, subscriber::Subscriber},
};
use dds_implementation::{
    data_representation_builtin_endpoints::{
        sedp_discovered_reader_data::{SedpDiscoveredReaderData, DCPS_SUBSCRIPTION},
        sedp_discovered_topic_data::{SedpDiscoveredTopicData, DCPS_TOPIC},
        sedp_discovered_writer_data::{SedpDiscoveredWriterData, DCPS_PUBLICATION},
        spdp_discovered_participant_data::{
            ParticipantProxy, SpdpDiscoveredParticipantData, DCPS_PARTICIPANT,
        },
    },
    dds_impl::{
        data_reader_proxy::RtpsReader,
        data_writer_proxy::RtpsWriter,
        domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
        publisher_proxy::PublisherProxy,
        subscriber_proxy::SubscriberProxy,
    },
    utils::shared_object::RtpsShared,
};
use rtps_implementation::{
    rtps_reader_proxy_impl::RtpsReaderProxyImpl, rtps_writer_proxy_impl::RtpsWriterProxyImpl,
};
use rtps_pim::{
    behavior::{
        reader::{
            stateful_reader::RtpsStatefulReaderOperations,
            writer_proxy::{RtpsWriterProxyAttributes, RtpsWriterProxyConstructor},
        },
        writer::{
            reader_proxy::{RtpsReaderProxyAttributes, RtpsReaderProxyConstructor},
            stateful_writer::RtpsStatefulWriterOperations,
        },
    },
    discovery::{
        participant_discovery::ParticipantDiscovery,
        spdp::spdp_discovered_participant_data::RtpsSpdpDiscoveredParticipantDataAttributes,
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    structure::{entity::RtpsEntityAttributes, participant::RtpsParticipantAttributes},
};

use crate::domain_participant_factory::RtpsStructureImpl;

pub struct Executor {
    pub receiver: Receiver<EnabledPeriodicTask>,
}

impl Executor {
    pub fn run(&self) {
        while let Ok(mut enabled_periodic_task) = self.receiver.try_recv() {
            async_std::task::spawn(async move {
                let mut interval = async_std::stream::interval(enabled_periodic_task.period);
                loop {
                    if enabled_periodic_task.enabled.load(atomic::Ordering::SeqCst) {
                        (enabled_periodic_task.task)();
                    } else {
                        println!("Task not enabled: {}", enabled_periodic_task.name);
                    }
                    interval.next().await;
                }
            });
        }
    }
}

#[derive(Clone)]
pub struct Spawner {
    pub task_sender: SyncSender<EnabledPeriodicTask>,
    pub enabled: Arc<AtomicBool>,
}

impl Spawner {
    pub fn new(task_sender: SyncSender<EnabledPeriodicTask>) -> Self {
        Self {
            task_sender,
            enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn spawn_enabled_periodic_task(
        &self,
        name: &'static str,
        task: impl FnMut() -> () + Send + Sync + 'static,
        period: std::time::Duration,
    ) {
        self.task_sender
            .send(EnabledPeriodicTask {
                name,
                task: Box::new(task),
                period,
                enabled: self.enabled.clone(),
            })
            .unwrap();
    }

    pub fn enable_tasks(&self) {
        self.enabled.store(true, atomic::Ordering::SeqCst);
    }

    pub fn _disable_tasks(&self) {
        self.enabled.store(false, atomic::Ordering::SeqCst);
    }
}

pub struct EnabledPeriodicTask {
    pub name: &'static str,
    pub task: Box<dyn FnMut() -> () + Send + Sync>,
    pub period: std::time::Duration,
    pub enabled: Arc<AtomicBool>,
}

pub fn task_spdp_discovery(
    domain_participant: RtpsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
) -> DDSResult<()> {
    let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());
    let builtin_subscriber = SubscriberProxy::new(
        domain_participant_proxy.clone(),
        domain_participant
            .read_lock()
            .builtin_subscriber
            .as_ref()
            .ok_or(DDSError::PreconditionNotMet(
                "Domain participant has no builtin subscriber".to_string(),
            ))?
            .downgrade(),
    );
    let builtin_publisher = PublisherProxy::new(
        domain_participant
            .read_lock()
            .builtin_publisher
            .as_ref()
            .ok_or(DDSError::PreconditionNotMet(
                "Domain participant has no builtin publisher".to_string(),
            ))?
            .downgrade(),
    );

    let dcps_participant_topic = domain_participant_proxy
        .lookup_topicdescription::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)?;
    let dcps_publication_topic = domain_participant_proxy
        .lookup_topicdescription::<SedpDiscoveredWriterData>(DCPS_PUBLICATION)?;
    let dcps_subscription_topic = domain_participant_proxy
        .lookup_topicdescription::<SedpDiscoveredReaderData>(DCPS_SUBSCRIPTION)?;
    let dcps_topic_topic =
        domain_participant_proxy.lookup_topicdescription::<SedpDiscoveredTopicData>(DCPS_TOPIC)?;

    let spdp_builtin_participant_data_reader =
        builtin_subscriber.lookup_datareader(&dcps_participant_topic)?;

    let sedp_builtin_publication_writer_shared = builtin_publisher
        .lookup_datawriter(&dcps_publication_topic)?
        .as_ref()
        .upgrade()?;

    let sedp_builtin_publication_reader_shared = builtin_subscriber
        .lookup_datareader(&dcps_publication_topic)?
        .as_ref()
        .upgrade()?;

    let sedp_builtin_subscription_writer_shared = builtin_publisher
        .lookup_datawriter(&dcps_subscription_topic)?
        .as_ref()
        .upgrade()?;

    let sedp_builtin_subscription_reader_shared = builtin_subscriber
        .lookup_datareader(&dcps_subscription_topic)?
        .as_ref()
        .upgrade()?;

    let sedp_builtin_topic_writer_shared = builtin_publisher
        .lookup_datawriter(&dcps_topic_topic)?
        .as_ref()
        .upgrade()?;

    let sedp_builtin_topic_reader_shared = builtin_subscriber
        .lookup_datareader(&dcps_topic_topic)?
        .as_ref()
        .upgrade()?;

    let mut sedp_builtin_publication_writer_lock =
        sedp_builtin_publication_writer_shared.write_lock();
    let sedp_builtin_publication_writer = sedp_builtin_publication_writer_lock
        .rtps_writer
        .try_as_stateful_writer()?;

    let mut sedp_builtin_publication_reader_lock =
        sedp_builtin_publication_reader_shared.write_lock();
    let sedp_builtin_publication_reader = sedp_builtin_publication_reader_lock
        .rtps_reader
        .try_as_stateful_reader()?;

    let mut sedp_builtin_subscription_writer_lock =
        sedp_builtin_subscription_writer_shared.write_lock();
    let sedp_builtin_subscription_writer = sedp_builtin_subscription_writer_lock
        .rtps_writer
        .try_as_stateful_writer()?;

    let mut sedp_builtin_subscription_reader_lock =
        sedp_builtin_subscription_reader_shared.write_lock();
    let sedp_builtin_subscription_reader = sedp_builtin_subscription_reader_lock
        .rtps_reader
        .try_as_stateful_reader()?;

    let mut sedp_builtin_topic_writer_lock = sedp_builtin_topic_writer_shared.write_lock();
    let sedp_builtin_topic_writer = sedp_builtin_topic_writer_lock
        .rtps_writer
        .try_as_stateful_writer()?;

    let mut sedp_builtin_topic_reader_lock = sedp_builtin_topic_reader_shared.write_lock();
    let sedp_builtin_topic_reader = sedp_builtin_topic_reader_lock
        .rtps_reader
        .try_as_stateful_reader()?;

    if let Ok(samples) = spdp_builtin_participant_data_reader.take(1, &[], &[], &[]) {
        for (discovered_participant, _) in samples.iter() {
            if let Ok(participant_discovery) = ParticipantDiscovery::new(
                discovered_participant,
                domain_participant.read_lock().domain_id as u32,
                &domain_participant.read_lock().domain_tag,
            ) {
                if !sedp_builtin_publication_writer
                    .matched_readers
                    .iter()
                    .any(|r| r.remote_reader_guid().prefix == discovered_participant.guid_prefix())
                {
                    participant_discovery.discovered_participant_add_publications_writer(
                        sedp_builtin_publication_writer,
                    );
                }

                if !sedp_builtin_publication_reader
                    .matched_writers
                    .iter()
                    .any(|w| w.remote_writer_guid().prefix == discovered_participant.guid_prefix())
                {
                    participant_discovery.discovered_participant_add_publications_reader(
                        sedp_builtin_publication_reader,
                    );
                }

                if !sedp_builtin_subscription_writer
                    .matched_readers
                    .iter()
                    .any(|r| r.remote_reader_guid().prefix == discovered_participant.guid_prefix())
                {
                    participant_discovery.discovered_participant_add_subscriptions_writer(
                        sedp_builtin_subscription_writer,
                    );
                }

                if !sedp_builtin_subscription_reader
                    .matched_writers
                    .iter()
                    .any(|w| w.remote_writer_guid().prefix == discovered_participant.guid_prefix())
                {
                    participant_discovery.discovered_participant_add_subscriptions_reader(
                        sedp_builtin_subscription_reader,
                    );
                }

                if !sedp_builtin_topic_writer
                    .matched_readers
                    .iter()
                    .any(|r| r.remote_reader_guid().prefix == discovered_participant.guid_prefix())
                {
                    participant_discovery
                        .discovered_participant_add_topics_writer(sedp_builtin_topic_writer);
                }

                if !sedp_builtin_topic_reader
                    .matched_writers
                    .iter()
                    .any(|w| w.remote_writer_guid().prefix == discovered_participant.guid_prefix())
                {
                    participant_discovery
                        .discovered_participant_add_topics_reader(sedp_builtin_topic_reader);
                }
            }
        }
    }

    Ok(())
}

pub fn task_sedp_writer_discovery(
    domain_participant: RtpsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
) -> DDSResult<()> {
    if domain_participant
        .read_lock()
        .user_defined_subscriber_list
        .is_empty()
    {
        return Ok(());
    }

    let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

    let builtin_subscriber = SubscriberProxy::new(
        domain_participant_proxy.clone(),
        domain_participant
            .read_lock()
            .builtin_subscriber
            .as_ref()
            .ok_or(DDSError::PreconditionNotMet(
                "Domain participant has no builtin subscriber".to_string(),
            ))?
            .downgrade(),
    );
    let dcps_publication_topic = domain_participant_proxy
        .lookup_topicdescription::<SedpDiscoveredWriterData>(DCPS_PUBLICATION)?;
    let sedp_builtin_publication_reader =
        builtin_subscriber.lookup_datareader(&dcps_publication_topic)?;

    let samples = sedp_builtin_publication_reader.take(1, &[], &[], &[]);

    for (sample, _) in samples.unwrap_or(vec![]).iter() {
        let topic_name = &sample.publication_builtin_topic_data.topic_name;
        let type_name = &sample.publication_builtin_topic_data.type_name;
        for subscriber in domain_participant
            .read_lock()
            .user_defined_subscriber_list
            .iter()
        {
            let subscriber_lock = subscriber.read_lock();
            for data_reader in subscriber_lock.data_reader_list.iter() {
                let mut data_reader_lock = data_reader.write_lock();
                let reader_topic_name = &data_reader_lock.topic.read_lock().topic_name.clone();
                let reader_type_name = data_reader_lock.topic.read_lock().type_name;
                if topic_name == reader_topic_name && type_name == reader_type_name {
                    let writer_proxy = RtpsWriterProxyImpl::new(
                        sample.writer_proxy.remote_writer_guid,
                        sample.writer_proxy.unicast_locator_list.as_ref(),
                        sample.writer_proxy.multicast_locator_list.as_ref(),
                        sample.writer_proxy.data_max_size_serialized,
                        sample.writer_proxy.remote_group_entity_id,
                    );
                    match &mut data_reader_lock.rtps_reader {
                        RtpsReader::Stateless(_) => (),
                        RtpsReader::Stateful(rtps_stateful_reader) => {
                            rtps_stateful_reader.matched_writer_add(writer_proxy);

                            data_reader_lock.status.total_count += 1;
                            data_reader_lock.status.total_count_change += 1;
                            data_reader_lock.status.current_count += 1;
                            data_reader_lock.status.current_count_change += 1;

                            data_reader_lock.listener.as_ref().map(|l| {
                                l.trigger_on_subscription_matched(data_reader_lock.status)
                            });

                            data_reader_lock.status.total_count_change = 0;
                            data_reader_lock.status.current_count_change = 0;
                        }
                    };
                }
            }
        }
    }

    Ok(())
}

pub fn task_sedp_reader_discovery(
    domain_participant: RtpsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
) -> DDSResult<()> {
    if domain_participant
        .read_lock()
        .user_defined_publisher_list
        .is_empty()
    {
        return Ok(());
    }

    let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

    let builtin_subscriber = SubscriberProxy::new(
        domain_participant_proxy.clone(),
        domain_participant
            .read_lock()
            .builtin_subscriber
            .as_ref()
            .ok_or(DDSError::PreconditionNotMet(
                "Domain participant has no builtin subscriber".to_string(),
            ))?
            .downgrade(),
    );
    let dcps_subscription_topic = domain_participant_proxy
        .lookup_topicdescription::<SedpDiscoveredReaderData>(DCPS_SUBSCRIPTION)?;
    let sedp_builtin_subscription_reader =
        builtin_subscriber.lookup_datareader(&dcps_subscription_topic)?;

    let samples = sedp_builtin_subscription_reader.take(1, &[], &[], &[]);

    for (sample, _) in samples.unwrap_or(vec![]).iter() {
        let topic_name = &sample.subscription_builtin_topic_data.topic_name;
        let type_name = &sample.subscription_builtin_topic_data.type_name;
        for publisher in domain_participant
            .read_lock()
            .user_defined_publisher_list
            .iter()
        {
            let publisher_lock = publisher.read_lock();
            for data_writer in publisher_lock.data_writer_list.iter() {
                let mut data_writer_lock = data_writer.write_lock();
                let writer_topic_name = &data_writer_lock.topic.read_lock().topic_name.clone();
                let writer_type_name = data_writer_lock.topic.read_lock().type_name;
                if topic_name == writer_topic_name && type_name == writer_type_name {
                    let reader_proxy = RtpsReaderProxyImpl::new(
                        sample.reader_proxy.remote_reader_guid,
                        sample.reader_proxy.remote_group_entity_id,
                        sample.reader_proxy.unicast_locator_list.as_ref(),
                        sample.reader_proxy.multicast_locator_list.as_ref(),
                        sample.reader_proxy.expects_inline_qos,
                        true, // ???
                    );
                    match &mut data_writer_lock.rtps_writer {
                        RtpsWriter::Stateless(_) => (),
                        RtpsWriter::Stateful(rtps_stateful_writer) => {
                            rtps_stateful_writer.matched_reader_add(reader_proxy);

                            data_writer_lock.status.total_count += 1;

                            data_writer_lock
                                .listener
                                .as_ref()
                                .map(|l| l.on_publication_matched(data_writer_lock.status));
                        }
                    };
                }
            }
        }
    }

    Ok(())
}

pub fn task_announce_participant(
    domain_participant: RtpsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
) -> DDSResult<()> {
    let spdp_participant_writer = {
        let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());
        let dcps_topic_participant =
            domain_participant_proxy.lookup_topicdescription(DCPS_PARTICIPANT)?;
        let builtin_publisher = PublisherProxy::new(
            domain_participant
                .read_lock()
                .builtin_publisher
                .as_ref()
                .ok_or(DDSError::PreconditionNotMet(
                    "Domain participant has no builtin publisher".to_string(),
                ))?
                .downgrade(),
        );

        builtin_publisher.lookup_datawriter(&dcps_topic_participant)?
    };

    let discovered_participant_data = SpdpDiscoveredParticipantData {
        dds_participant_data: ParticipantBuiltinTopicData {
            key: BuiltInTopicKey {
                value: domain_participant
                    .read_lock()
                    .rtps_participant
                    .guid()
                    .into(),
            },
            user_data: domain_participant.read_lock().qos.user_data.clone(),
        },
        participant_proxy: ParticipantProxy {
            domain_id: domain_participant.read_lock().domain_id as u32,
            domain_tag: domain_participant.read_lock().domain_tag.clone(),
            protocol_version: domain_participant
                .read_lock()
                .rtps_participant
                .protocol_version(),
            guid_prefix: domain_participant
                .read_lock()
                .rtps_participant
                .guid()
                .prefix(),
            vendor_id: domain_participant.read_lock().rtps_participant.vendor_id(),
            expects_inline_qos: false,
            metatraffic_unicast_locator_list: domain_participant
                .read_lock()
                .metatraffic_unicast_locator_list
                .clone(),
            metatraffic_multicast_locator_list: domain_participant
                .read_lock()
                .metatraffic_multicast_locator_list
                .clone(),
            default_unicast_locator_list: domain_participant
                .read_lock()
                .rtps_participant
                .default_unicast_locator_list()
                .to_vec(),
            default_multicast_locator_list: domain_participant
                .read_lock()
                .rtps_participant
                .default_multicast_locator_list()
                .to_vec(),
            available_builtin_endpoints: BuiltinEndpointSet::default(),
            manual_liveliness_count: domain_participant.read_lock().manual_liveliness_count,
            builtin_endpoint_qos: BuiltinEndpointQos::default(),
        },
        lease_duration: domain_participant.read_lock().lease_duration,
    };

    spdp_participant_writer.write_w_timestamp(
        &discovered_participant_data,
        None,
        Time { sec: 0, nanosec: 0 },
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use dds_api::{
        domain::domain_participant::DomainParticipant,
        infrastructure::qos::DomainParticipantQos,
        publication::publisher::Publisher,
        return_type::{DDSError, DDSResult},
        subscription::subscriber::Subscriber,
    };
    use dds_implementation::{
        data_representation_builtin_endpoints::{
            sedp_discovered_reader_data::{SedpDiscoveredReaderData, DCPS_SUBSCRIPTION},
            sedp_discovered_topic_data::{SedpDiscoveredTopicData, DCPS_TOPIC},
            sedp_discovered_writer_data::{SedpDiscoveredWriterData, DCPS_PUBLICATION},
        },
        dds_impl::{
            domain_participant_proxy::{DomainParticipantAttributes, DomainParticipantProxy},
            publisher_proxy::PublisherProxy,
            subscriber_proxy::SubscriberProxy,
        },
        dds_type::{DdsDeserialize, DdsSerialize, DdsType},
        utils::shared_object::RtpsShared,
    };

    use crate::{
        domain_participant_factory::{create_builtins, Communications, RtpsStructureImpl},
        tasks::task_sedp_reader_discovery,
    };

    use super::{task_announce_participant, task_sedp_writer_discovery, task_spdp_discovery};

    struct UserData(u8);

    impl DdsType for UserData {
        fn type_name() -> &'static str {
            "UserData"
        }

        fn has_key() -> bool {
            false
        }
    }

    impl DdsSerialize for UserData {
        fn serialize<W: std::io::Write, E: dds_implementation::dds_type::Endianness>(
            &self,
            mut writer: W,
        ) -> DDSResult<()> {
            writer
                .write(&[self.0])
                .map(|_| ())
                .map_err(|e| DDSError::PreconditionNotMet(format!("{}", e)))
        }
    }

    impl<'de> DdsDeserialize<'de> for UserData {
        fn deserialize(buf: &mut &'de [u8]) -> DDSResult<Self> {
            Ok(UserData(buf[0]))
        }
    }

    macro_rules! matched_readers {
        ($writer:ident) => {
            $writer
                .as_ref()
                .upgrade()
                .unwrap()
                .write_lock()
                .rtps_writer
                .try_as_stateful_writer()
                .unwrap()
                .matched_readers
        };
    }

    macro_rules! matched_writers {
        ($reader:ident) => {
            $reader
                .as_ref()
                .upgrade()
                .unwrap()
                .write_lock()
                .rtps_reader
                .try_as_stateful_reader()
                .unwrap()
                .matched_writers
        };
    }

    #[test]
    fn spdp_task_matches_all_sedp_endpoints() {
        let domain_id = 1;
        let multicast_address = [239, 255, 0, 1];
        let unicast_address = [127, 0, 0, 1];

        // Create 2 participants
        let mut communications1 = Communications::find_available(
            domain_id,
            [0; 6],
            unicast_address.into(),
            multicast_address.into(),
        )
        .unwrap();
        let participant1 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications1.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![communications1.metatraffic_unicast_locator()],
            vec![communications1.metatraffic_multicast_locator()],
            vec![communications1.default_unicast_locator()],
            vec![],
        ));
        create_builtins(participant1.clone()).unwrap();
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());

        let mut communications2 = Communications::find_available(
            domain_id,
            [0; 6],
            unicast_address.into(),
            multicast_address.into(),
        )
        .unwrap();
        let participant2 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications2.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![communications2.metatraffic_unicast_locator()],
            vec![communications2.metatraffic_multicast_locator()],
            vec![communications2.default_unicast_locator()],
            vec![],
        ));
        create_builtins(participant2.clone()).unwrap();
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());

        // Get the SEDP endpoints
        let dcps_publication_topic = participant1_proxy
            .lookup_topicdescription::<SedpDiscoveredWriterData>(DCPS_PUBLICATION)
            .unwrap();
        let dcps_subscription_topic = participant1_proxy
            .lookup_topicdescription::<SedpDiscoveredReaderData>(DCPS_SUBSCRIPTION)
            .unwrap();
        let dcps_topic_topic = participant1_proxy
            .lookup_topicdescription::<SedpDiscoveredTopicData>(DCPS_TOPIC)
            .unwrap();

        let participant1_publisher = PublisherProxy::new(
            participant1
                .read_lock()
                .builtin_publisher
                .as_ref()
                .unwrap()
                .downgrade(),
        );
        let participant1_subscriber = SubscriberProxy::new(
            participant1_proxy.clone(),
            participant1
                .read_lock()
                .builtin_subscriber
                .as_ref()
                .unwrap()
                .downgrade(),
        );

        let participant1_sedp_publication_writer = participant1_publisher
            .lookup_datawriter(&dcps_publication_topic)
            .unwrap();
        let participant1_sedp_publication_reader = participant1_subscriber
            .lookup_datareader(&dcps_publication_topic)
            .unwrap();
        let participant1_sedp_subscription_writer = participant1_publisher
            .lookup_datawriter(&dcps_subscription_topic)
            .unwrap();
        let participant1_sedp_subscription_reader = participant1_subscriber
            .lookup_datareader(&dcps_subscription_topic)
            .unwrap();
        let participant1_sedp_topic_writer = participant1_publisher
            .lookup_datawriter(&dcps_topic_topic)
            .unwrap();
        let participant1_sedp_topic_reader = participant1_subscriber
            .lookup_datareader(&dcps_topic_topic)
            .unwrap();

        let participant2_publisher = PublisherProxy::new(
            participant2
                .read_lock()
                .builtin_publisher
                .as_ref()
                .unwrap()
                .downgrade(),
        );
        let participant2_subscriber = SubscriberProxy::new(
            participant2_proxy.clone(),
            participant2
                .read_lock()
                .builtin_subscriber
                .as_ref()
                .unwrap()
                .downgrade(),
        );

        let participant2_sedp_publication_writer = participant2_publisher
            .lookup_datawriter(&dcps_publication_topic)
            .unwrap();
        let participant2_sedp_publication_reader = participant2_subscriber
            .lookup_datareader(&dcps_publication_topic)
            .unwrap();
        let participant2_sedp_subscription_writer = participant2_publisher
            .lookup_datawriter(&dcps_subscription_topic)
            .unwrap();
        let participant2_sedp_subscription_reader = participant2_subscriber
            .lookup_datareader(&dcps_subscription_topic)
            .unwrap();
        let participant2_sedp_topic_writer = participant2_publisher
            .lookup_datawriter(&dcps_topic_topic)
            .unwrap();
        let participant2_sedp_topic_reader = participant2_subscriber
            .lookup_datareader(&dcps_topic_topic)
            .unwrap();

        // Before the SPDP task nothing is matched
        {
            assert_eq!(
                0,
                matched_readers!(participant1_sedp_publication_writer).len()
            );
            assert_eq!(
                0,
                matched_writers!(participant1_sedp_publication_reader).len()
            );
            assert_eq!(
                0,
                matched_readers!(participant1_sedp_subscription_writer).len()
            );
            assert_eq!(
                0,
                matched_writers!(participant1_sedp_subscription_reader).len()
            );
            assert_eq!(0, matched_readers!(participant1_sedp_topic_writer).len());
            assert_eq!(0, matched_writers!(participant1_sedp_topic_reader).len());
            assert_eq!(
                0,
                matched_readers!(participant2_sedp_publication_writer).len()
            );
            assert_eq!(
                0,
                matched_writers!(participant2_sedp_publication_reader).len()
            );
            assert_eq!(
                0,
                matched_readers!(participant2_sedp_subscription_writer).len()
            );
            assert_eq!(
                0,
                matched_writers!(participant2_sedp_subscription_reader).len()
            );
            assert_eq!(0, matched_readers!(participant2_sedp_topic_writer).len());
            assert_eq!(0, matched_writers!(participant2_sedp_topic_reader).len());
        }

        // Announce the participant
        {
            task_announce_participant(participant1.clone()).unwrap();
            task_announce_participant(participant2.clone()).unwrap();

            communications1
                .metatraffic_unicast
                .send(core::slice::from_ref(
                    participant1.read_lock().builtin_publisher.as_ref().unwrap(),
                ));
            communications2
                .metatraffic_unicast
                .send(core::slice::from_ref(
                    participant2.read_lock().builtin_publisher.as_ref().unwrap(),
                ));

            communications1
                .metatraffic_multicast
                .receive(core::slice::from_ref(
                    participant1
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()
                        .unwrap(),
                ));
            communications2
                .metatraffic_multicast
                .receive(core::slice::from_ref(
                    participant2
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()
                        .unwrap(),
                ));
        }

        task_spdp_discovery(participant1.clone()).unwrap();
        task_spdp_discovery(participant2.clone()).unwrap();

        // After the SPDP task everything is matched to 2 endpoints (from itself and the other participant)
        {
            assert_eq!(
                2,
                matched_readers!(participant1_sedp_publication_writer).len()
            );
            assert_eq!(
                2,
                matched_writers!(participant1_sedp_publication_reader).len()
            );
            assert_eq!(
                2,
                matched_readers!(participant1_sedp_subscription_writer).len()
            );
            assert_eq!(
                2,
                matched_writers!(participant1_sedp_subscription_reader).len()
            );
            assert_eq!(2, matched_readers!(participant1_sedp_topic_writer).len());
            assert_eq!(2, matched_writers!(participant1_sedp_topic_reader).len());
            assert_eq!(
                2,
                matched_readers!(participant2_sedp_publication_writer).len()
            );
            assert_eq!(
                2,
                matched_writers!(participant2_sedp_publication_reader).len()
            );
            assert_eq!(
                2,
                matched_readers!(participant2_sedp_subscription_writer).len()
            );
            assert_eq!(
                2,
                matched_writers!(participant2_sedp_subscription_reader).len()
            );
            assert_eq!(2, matched_readers!(participant2_sedp_topic_writer).len());
            assert_eq!(2, matched_writers!(participant2_sedp_topic_reader).len());
        }
    }

    #[test]
    fn sedp_discovery_matches_user_readers_and_writers() {
        let domain_id = 2;
        let multicast_address = [239, 255, 0, 1];
        let unicast_address = [127, 0, 0, 1];

        // Create 2 participants
        let mut communications1 = Communications::find_available(
            domain_id,
            [0; 6],
            unicast_address.into(),
            multicast_address.into(),
        )
        .unwrap();
        let participant1 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications1.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![communications1.metatraffic_unicast_locator()],
            vec![communications1.metatraffic_multicast_locator()],
            vec![communications1.default_unicast_locator()],
            vec![],
        ));
        create_builtins(participant1.clone()).unwrap();
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());

        let mut communications2 = Communications::find_available(
            domain_id,
            [0; 6],
            unicast_address.into(),
            multicast_address.into(),
        )
        .unwrap();
        let participant2 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications2.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![communications2.metatraffic_unicast_locator()],
            vec![communications2.metatraffic_multicast_locator()],
            vec![communications2.default_unicast_locator()],
            vec![],
        ));
        create_builtins(participant2.clone()).unwrap();
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());

        // Match SEDP endpoints
        {
            task_announce_participant(participant1.clone()).unwrap();
            task_announce_participant(participant2.clone()).unwrap();

            communications1
                .metatraffic_unicast
                .send(core::slice::from_ref(
                    participant1.read_lock().builtin_publisher.as_ref().unwrap(),
                ));
            communications2
                .metatraffic_unicast
                .send(core::slice::from_ref(
                    participant2.read_lock().builtin_publisher.as_ref().unwrap(),
                ));

            communications1
                .metatraffic_multicast
                .receive(core::slice::from_ref(
                    participant1
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()
                        .unwrap(),
                ));
            communications2
                .metatraffic_multicast
                .receive(core::slice::from_ref(
                    participant2
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()
                        .unwrap(),
                ));

            task_spdp_discovery(participant1.clone()).unwrap();
            task_spdp_discovery(participant2.clone()).unwrap();
        }

        // Create a reader and a writer on each participants
        let my_topic = participant1_proxy
            .create_topic::<UserData>("MyTopic", None, None, 0)
            .unwrap();
        let publisher1 = participant1_proxy.create_publisher(None, None, 0).unwrap();
        let writer1 = publisher1
            .create_datawriter(&my_topic, None, None, 0)
            .unwrap();
        let subscriber1 = participant1_proxy.create_subscriber(None, None, 0).unwrap();
        let reader1 = subscriber1
            .create_datareader(&my_topic, None, None, 0)
            .unwrap();

        let publisher2 = participant2_proxy.create_publisher(None, None, 0).unwrap();
        let writer2 = publisher2
            .create_datawriter(&my_topic, None, None, 0)
            .unwrap();
        let subscriber2 = participant2_proxy.create_subscriber(None, None, 0).unwrap();
        let reader2 = subscriber2
            .create_datareader(&my_topic, None, None, 0)
            .unwrap();

        // Send SEDP data
        {
            communications1
                .metatraffic_unicast
                .send(core::slice::from_ref(
                    participant1.read_lock().builtin_publisher.as_ref().unwrap(),
                ));
            communications2
                .metatraffic_unicast
                .send(core::slice::from_ref(
                    participant2.read_lock().builtin_publisher.as_ref().unwrap(),
                ));

            communications1
                .metatraffic_unicast
                .receive(core::slice::from_ref(
                    participant1
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()
                        .unwrap(),
                ));
            communications2
                .metatraffic_unicast
                .receive(core::slice::from_ref(
                    participant2
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()
                        .unwrap(),
                ));
        }

        // Before the SEDP task the readers/writers are not matched
        {
            assert_eq!(0, matched_readers!(writer1).len());
            assert_eq!(0, matched_readers!(writer2).len());
            assert_eq!(0, matched_writers!(reader1).len());
            assert_eq!(0, matched_writers!(reader2).len());
        }

        // call SEDP task
        task_sedp_reader_discovery(participant1.clone()).unwrap();
        task_sedp_reader_discovery(participant2.clone()).unwrap();
        task_sedp_writer_discovery(participant1.clone()).unwrap();
        task_sedp_writer_discovery(participant2.clone()).unwrap();

        // After the SEDP task the readers/writers are matched
        {
            assert_eq!(2, matched_readers!(writer1).len());
            assert_eq!(2, matched_readers!(writer2).len());
            assert_eq!(2, matched_writers!(reader1).len());
            assert_eq!(2, matched_writers!(reader2).len());
        }
    }

    #[test]
    fn sedp_discovery_doesnt_match_different_topics() {
        let domain_id = 3;
        let multicast_address = [239, 255, 0, 1];
        let unicast_address = [127, 0, 0, 1];

        // Create 2 participants
        let mut communications1 = Communications::find_available(
            domain_id,
            [0; 6],
            unicast_address.into(),
            multicast_address.into(),
        )
        .unwrap();
        let participant1 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications1.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![communications1.metatraffic_unicast_locator()],
            vec![communications1.metatraffic_multicast_locator()],
            vec![communications1.default_unicast_locator()],
            vec![],
        ));
        create_builtins(participant1.clone()).unwrap();
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());

        let mut communications2 = Communications::find_available(
            domain_id,
            [0; 6],
            unicast_address.into(),
            multicast_address.into(),
        )
        .unwrap();
        let participant2 = RtpsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications2.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            vec![communications2.metatraffic_unicast_locator()],
            vec![communications2.metatraffic_multicast_locator()],
            vec![communications2.default_unicast_locator()],
            vec![],
        ));
        create_builtins(participant2.clone()).unwrap();
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());

        // Match SEDP endpoints
        {
            task_announce_participant(participant1.clone()).unwrap();
            task_announce_participant(participant2.clone()).unwrap();

            communications1
                .metatraffic_unicast
                .send(core::slice::from_ref(
                    participant1.read_lock().builtin_publisher.as_ref().unwrap(),
                ));
            communications1
                .metatraffic_unicast
                .send(core::slice::from_ref(
                    participant2.read_lock().builtin_publisher.as_ref().unwrap(),
                ));

            communications1
                .metatraffic_multicast
                .receive(core::slice::from_ref(
                    participant1
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()
                        .unwrap(),
                ));
            communications2
                .metatraffic_multicast
                .receive(core::slice::from_ref(
                    participant2
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()
                        .unwrap(),
                ));

            task_spdp_discovery(participant1.clone()).unwrap();
            task_spdp_discovery(participant2.clone()).unwrap();
        }

        // Create a reader and a writer on each participants
        let my_topic1 = participant1_proxy
            .create_topic::<UserData>("MyTopic1", None, None, 0)
            .unwrap();
        let publisher1 = participant1_proxy.create_publisher(None, None, 0).unwrap();
        let writer1 = publisher1
            .create_datawriter(&my_topic1, None, None, 0)
            .unwrap();
        let subscriber1 = participant1_proxy.create_subscriber(None, None, 0).unwrap();
        let reader1 = subscriber1
            .create_datareader(&my_topic1, None, None, 0)
            .unwrap();

        let my_topic2 = participant2_proxy
            .create_topic::<UserData>("MyTopic2", None, None, 0)
            .unwrap();
        let publisher2 = participant2_proxy.create_publisher(None, None, 0).unwrap();
        let writer2 = publisher2
            .create_datawriter(&my_topic2, None, None, 0)
            .unwrap();
        let subscriber2 = participant2_proxy.create_subscriber(None, None, 0).unwrap();
        let reader2 = subscriber2
            .create_datareader(&my_topic2, None, None, 0)
            .unwrap();

        // Send SEDP data
        {
            communications1
                .metatraffic_unicast
                .send(core::slice::from_ref(
                    participant1.read_lock().builtin_publisher.as_ref().unwrap(),
                ));
            communications2
                .metatraffic_unicast
                .send(core::slice::from_ref(
                    participant2.read_lock().builtin_publisher.as_ref().unwrap(),
                ));

            communications1
                .metatraffic_unicast
                .receive(core::slice::from_ref(
                    participant1
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()
                        .unwrap(),
                ));
            communications2
                .metatraffic_unicast
                .receive(core::slice::from_ref(
                    participant2
                        .read_lock()
                        .builtin_subscriber
                        .as_ref()
                        .unwrap(),
                ));
        }

        // Before the SEDP task the readers/writers are not matched
        {
            assert_eq!(0, matched_readers!(writer1).len());
            assert_eq!(0, matched_readers!(writer2).len());
            assert_eq!(0, matched_writers!(reader1).len());
            assert_eq!(0, matched_writers!(reader2).len());
        }

        // call SEDP task
        task_sedp_reader_discovery(participant1.clone()).unwrap();
        task_sedp_reader_discovery(participant2.clone()).unwrap();
        task_sedp_writer_discovery(participant1.clone()).unwrap();
        task_sedp_writer_discovery(participant2.clone()).unwrap();

        // After the SEDP task the readers/writers are matched only on the same participant
        {
            assert_eq!(1, matched_readers!(writer1).len());
            assert_eq!(1, matched_readers!(writer2).len());
            assert_eq!(1, matched_writers!(reader1).len());
            assert_eq!(1, matched_writers!(reader2).len());
        }
    }
}
