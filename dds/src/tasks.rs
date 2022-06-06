use std::sync::{
    atomic::{self, AtomicBool},
    mpsc::{Receiver, SyncSender},
    Arc,
};

use async_std::prelude::StreamExt;
use dds_api::{
    builtin_topics::ParticipantBuiltinTopicData,
    dcps_psm::{BuiltInTopicKey, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
    domain::domain_participant::DomainParticipant,
    publication::{data_writer::DataWriter, publisher::Publisher},
    return_type::{DdsError, DdsResult},
    subscription::{data_reader::DataReader, subscriber::Subscriber},
};
use dds_implementation::{
    data_representation_builtin_endpoints::{
        discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
        discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
        spdp_discovered_participant_data::{
            ParticipantProxy, SpdpDiscoveredParticipantData, DCPS_PARTICIPANT,
        },
    },
    dds_impl::domain_participant_attributes::{
        AddDiscoveredParticipant, DomainParticipantAttributes,
    },
    utils::{
        discovery_traits::{AddMatchedReader, AddMatchedWriter},
        shared_object::DdsShared,
    },
};
use rtps_pim::{
    discovery::types::{BuiltinEndpointQos, BuiltinEndpointSet},
    structure::{entity::RtpsEntityAttributes, participant::RtpsParticipantAttributes},
};

use crate::{
    domain_participant_factory::RtpsStructureImpl,
    domain_participant_proxy::DomainParticipantProxy, publisher_proxy::PublisherProxy,
    subscriber_proxy::SubscriberProxy,
};

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
    domain_participant: DdsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
) -> DdsResult<()> {
    let builtin_subscriber = domain_participant.get_builtin_subscriber().unwrap();

    let dcps_participant_topic = domain_participant
        .lookup_topicdescription::<SpdpDiscoveredParticipantData>(DCPS_PARTICIPANT)?;

    let spdp_builtin_participant_data_reader = builtin_subscriber
        .lookup_datareader::<SpdpDiscoveredParticipantData>(&dcps_participant_topic)?;

    if let Ok(samples) = spdp_builtin_participant_data_reader.take(
        1,
        ANY_SAMPLE_STATE,
        ANY_VIEW_STATE,
        ANY_INSTANCE_STATE,
    ) {
        for (discovered_participant_data, _) in samples.iter() {
            domain_participant.add_discovered_participant(discovered_participant_data)
        }
    }

    Ok(())
}

pub fn task_sedp_writer_discovery(
    domain_participant: DdsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
) -> DdsResult<()> {
    if domain_participant
        .user_defined_subscriber_list
        .read_lock()
        .is_empty()
    {
        return Ok(());
    }

    let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

    let builtin_subscriber = SubscriberProxy::new(
        domain_participant
            .builtin_subscriber
            .read_lock()
            .as_ref()
            .ok_or(DdsError::PreconditionNotMet(
                "Domain participant has no builtin subscriber".to_string(),
            ))?
            .downgrade(),
    );
    let dcps_publication_topic = domain_participant_proxy
        .lookup_topicdescription::<DiscoveredWriterData>(DCPS_PUBLICATION)?;
    let sedp_builtin_publication_reader =
        builtin_subscriber.lookup_datareader(&dcps_publication_topic)?;

    let samples = sedp_builtin_publication_reader.take(
        1,
        ANY_SAMPLE_STATE,
        ANY_VIEW_STATE,
        ANY_INSTANCE_STATE,
    );

    for (sample, _) in samples.unwrap_or(vec![]).iter() {
        for subscriber in domain_participant
            .user_defined_subscriber_list
            .read_lock()
            .iter()
        {
            subscriber.add_matched_writer(&sample)
        }
    }

    Ok(())
}

pub fn task_sedp_reader_discovery(
    domain_participant: DdsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
) -> DdsResult<()> {
    if domain_participant
        .user_defined_publisher_list
        .read_lock()
        .is_empty()
    {
        return Ok(());
    }

    let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());

    let builtin_subscriber = SubscriberProxy::new(
        domain_participant
            .builtin_subscriber
            .read_lock()
            .as_ref()
            .ok_or(DdsError::PreconditionNotMet(
                "Domain participant has no builtin subscriber".to_string(),
            ))?
            .downgrade(),
    );
    let dcps_subscription_topic = domain_participant_proxy
        .lookup_topicdescription::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)?;
    let sedp_builtin_subscription_reader =
        builtin_subscriber.lookup_datareader(&dcps_subscription_topic)?;

    let samples = sedp_builtin_subscription_reader.take(
        1,
        ANY_SAMPLE_STATE,
        ANY_VIEW_STATE,
        ANY_INSTANCE_STATE,
    );

    for (sample, _) in samples.unwrap_or(vec![]).iter() {
        for publisher in domain_participant
            .user_defined_publisher_list
            .read_lock()
            .iter()
        {
            publisher.add_matched_reader(sample)
        }
    }

    Ok(())
}

pub fn task_announce_participant(
    domain_participant: DdsShared<DomainParticipantAttributes<RtpsStructureImpl>>,
) -> DdsResult<()> {
    let spdp_participant_writer = {
        let domain_participant_proxy = DomainParticipantProxy::new(domain_participant.downgrade());
        let dcps_topic_participant =
            domain_participant_proxy.lookup_topicdescription(DCPS_PARTICIPANT)?;
        let builtin_publisher = PublisherProxy::new(
            domain_participant
                .builtin_publisher
                .read_lock()
                .as_ref()
                .ok_or(DdsError::PreconditionNotMet(
                    "Domain participant has no builtin publisher".to_string(),
                ))?
                .downgrade(),
        );

        builtin_publisher.lookup_datawriter(&dcps_topic_participant)?
    };

    let discovered_participant_data = SpdpDiscoveredParticipantData {
        dds_participant_data: ParticipantBuiltinTopicData {
            key: BuiltInTopicKey {
                value: domain_participant.rtps_participant.guid().into(),
            },
            user_data: domain_participant.qos.user_data.clone(),
        },
        participant_proxy: ParticipantProxy {
            domain_id: domain_participant.domain_id as u32,
            domain_tag: domain_participant.domain_tag.clone(),
            protocol_version: domain_participant.rtps_participant.protocol_version(),
            guid_prefix: domain_participant.rtps_participant.guid().prefix(),
            vendor_id: domain_participant.rtps_participant.vendor_id(),
            expects_inline_qos: false,
            metatraffic_unicast_locator_list: domain_participant
                .metatraffic_unicast_locator_list
                .clone(),
            metatraffic_multicast_locator_list: domain_participant
                .metatraffic_multicast_locator_list
                .clone(),
            default_unicast_locator_list: domain_participant
                .rtps_participant
                .default_unicast_locator_list()
                .to_vec(),
            default_multicast_locator_list: domain_participant
                .rtps_participant
                .default_multicast_locator_list()
                .to_vec(),
            available_builtin_endpoints: BuiltinEndpointSet::default(),
            manual_liveliness_count: domain_participant.manual_liveliness_count,
            builtin_endpoint_qos: BuiltinEndpointQos::default(),
        },
        lease_duration: domain_participant.lease_duration,
    };

    spdp_participant_writer.write(&discovered_participant_data, None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use dds_api::{
        domain::domain_participant::DomainParticipant,
        infrastructure::qos::DomainParticipantQos,
        publication::{data_writer::DataWriter, publisher::Publisher},
        return_type::{DdsError, DdsResult},
        subscription::{data_reader::DataReader, subscriber::Subscriber},
    };
    use dds_implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
            discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
            discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
        },
        dds_impl::domain_participant_attributes::DomainParticipantAttributes,
        dds_type::{DdsDeserialize, DdsSerialize, DdsType},
        utils::shared_object::DdsShared,
    };

    use crate::{
        domain_participant_factory::{create_builtins, Communications, RtpsStructureImpl},
        domain_participant_proxy::DomainParticipantProxy,
        publisher_proxy::PublisherProxy,
        subscriber_proxy::SubscriberProxy,
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
        ) -> DdsResult<()> {
            writer
                .write(&[self.0])
                .map(|_| ())
                .map_err(|e| DdsError::PreconditionNotMet(format!("{}", e)))
        }
    }

    impl<'de> DdsDeserialize<'de> for UserData {
        fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
            Ok(UserData(buf[0]))
        }
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
            vec![unicast_address.into()],
            multicast_address.into(),
        )
        .unwrap();
        let participant1 = DdsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications1.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            communications1.metatraffic_unicast_locator_list(),
            communications1.metatraffic_multicast_locator_list(),
            communications1.default_unicast_locator_list(),
            vec![],
        ));
        create_builtins(participant1.clone()).unwrap();
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());

        let mut communications2 = Communications::find_available(
            domain_id,
            [0; 6],
            vec![unicast_address.into()],
            multicast_address.into(),
        )
        .unwrap();
        let participant2 = DdsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications2.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            communications2.metatraffic_unicast_locator_list(),
            communications2.metatraffic_multicast_locator_list(),
            communications2.default_unicast_locator_list(),
            vec![],
        ));
        create_builtins(participant2.clone()).unwrap();
        let _participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());

        // Get the SEDP endpoints
        let dcps_publication_topic = participant1_proxy
            .lookup_topicdescription::<DiscoveredWriterData>(DCPS_PUBLICATION)
            .unwrap();
        let dcps_subscription_topic = participant1_proxy
            .lookup_topicdescription::<DiscoveredReaderData>(DCPS_SUBSCRIPTION)
            .unwrap();
        let dcps_topic_topic = participant1_proxy
            .lookup_topicdescription::<DiscoveredTopicData>(DCPS_TOPIC)
            .unwrap();

        let participant1_publisher = PublisherProxy::new(
            participant1
                .builtin_publisher
                .read_lock()
                .as_ref()
                .unwrap()
                .downgrade(),
        );
        let participant1_subscriber = SubscriberProxy::new(
            participant1
                .builtin_subscriber
                .read_lock()
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
                .builtin_publisher
                .read_lock()
                .as_ref()
                .unwrap()
                .downgrade(),
        );
        let participant2_subscriber = SubscriberProxy::new(
            participant2
                .builtin_subscriber
                .read_lock()
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
                participant1_sedp_publication_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                participant1_sedp_publication_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                participant1_sedp_subscription_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                participant1_sedp_subscription_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                participant1_sedp_topic_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                participant1_sedp_topic_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                participant2_sedp_publication_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                participant2_sedp_publication_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                participant2_sedp_subscription_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                participant2_sedp_subscription_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                participant2_sedp_topic_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                0,
                participant2_sedp_topic_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
        }

        // Announce the participant
        {
            task_announce_participant(participant1.clone()).unwrap();
            task_announce_participant(participant2.clone()).unwrap();

            communications1.metatraffic_unicast.send_publisher_message(
                participant1.builtin_publisher.read_lock().as_ref().unwrap(),
            );
            communications2.metatraffic_unicast.send_publisher_message(
                participant2.builtin_publisher.read_lock().as_ref().unwrap(),
            );

            communications1.metatraffic_multicast.receive(
                &[],
                core::slice::from_ref(
                    participant1
                        .builtin_subscriber
                        .read_lock()
                        .as_ref()
                        .unwrap(),
                ),
            );
            communications2.metatraffic_multicast.receive(
                &[],
                core::slice::from_ref(
                    participant2
                        .builtin_subscriber
                        .read_lock()
                        .as_ref()
                        .unwrap(),
                ),
            );
        }

        task_spdp_discovery(participant1.clone()).unwrap();
        task_spdp_discovery(participant2.clone()).unwrap();

        // After the SPDP task everything is matched to 2 endpoints (from itself and the other participant)
        {
            assert_eq!(
                2,
                participant1_sedp_publication_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                2,
                participant1_sedp_publication_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                2,
                participant1_sedp_subscription_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                2,
                participant1_sedp_subscription_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                2,
                participant1_sedp_topic_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                2,
                participant1_sedp_topic_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                2,
                participant2_sedp_publication_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                2,
                participant2_sedp_publication_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                2,
                participant2_sedp_subscription_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                2,
                participant2_sedp_subscription_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                2,
                participant2_sedp_topic_writer
                    .get_matched_subscriptions()
                    .unwrap()
                    .len()
            );
            assert_eq!(
                2,
                participant2_sedp_topic_reader
                    .get_matched_publications()
                    .unwrap()
                    .len()
            );
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
            vec![unicast_address.into()],
            multicast_address.into(),
        )
        .unwrap();
        let participant1 = DdsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications1.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            communications1.metatraffic_unicast_locator_list(),
            communications1.metatraffic_multicast_locator_list(),
            communications1.default_unicast_locator_list(),
            vec![],
        ));
        create_builtins(participant1.clone()).unwrap();
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());

        let mut communications2 = Communications::find_available(
            domain_id,
            [0; 6],
            vec![unicast_address.into()],
            multicast_address.into(),
        )
        .unwrap();
        let participant2 = DdsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications2.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            communications2.metatraffic_unicast_locator_list(),
            communications2.metatraffic_multicast_locator_list(),
            communications2.default_unicast_locator_list(),
            vec![],
        ));
        create_builtins(participant2.clone()).unwrap();
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());

        // Match SEDP endpoints
        {
            task_announce_participant(participant1.clone()).unwrap();
            task_announce_participant(participant2.clone()).unwrap();

            communications1.metatraffic_unicast.send_publisher_message(
                participant1.builtin_publisher.read_lock().as_ref().unwrap(),
            );
            communications2.metatraffic_unicast.send_publisher_message(
                participant2.builtin_publisher.read_lock().as_ref().unwrap(),
            );

            communications1.metatraffic_multicast.receive(
                &[],
                core::slice::from_ref(
                    participant1
                        .builtin_subscriber
                        .read_lock()
                        .as_ref()
                        .unwrap(),
                ),
            );
            communications2.metatraffic_multicast.receive(
                &[],
                core::slice::from_ref(
                    participant2
                        .builtin_subscriber
                        .read_lock()
                        .as_ref()
                        .unwrap(),
                ),
            );

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
            communications1.metatraffic_unicast.send_publisher_message(
                participant1.builtin_publisher.read_lock().as_ref().unwrap(),
            );
            communications2.metatraffic_unicast.send_publisher_message(
                participant2.builtin_publisher.read_lock().as_ref().unwrap(),
            );

            communications1.metatraffic_unicast.receive(
                &[],
                core::slice::from_ref(
                    participant1
                        .builtin_subscriber
                        .read_lock()
                        .as_ref()
                        .unwrap(),
                ),
            );
            communications2.metatraffic_unicast.receive(
                &[],
                core::slice::from_ref(
                    participant2
                        .builtin_subscriber
                        .read_lock()
                        .as_ref()
                        .unwrap(),
                ),
            );
        }

        // Before the SEDP task the readers/writers are not matched
        {
            assert_eq!(0, writer1.get_matched_subscriptions().unwrap().len());
            assert_eq!(0, writer2.get_matched_subscriptions().unwrap().len());
            assert_eq!(0, reader1.get_matched_publications().unwrap().len());
            assert_eq!(0, reader2.get_matched_publications().unwrap().len());
        }

        // call SEDP task
        task_sedp_reader_discovery(participant1.clone()).unwrap();
        task_sedp_reader_discovery(participant2.clone()).unwrap();
        task_sedp_writer_discovery(participant1.clone()).unwrap();
        task_sedp_writer_discovery(participant2.clone()).unwrap();

        // After the SEDP task the readers/writers are matched
        {
            assert_eq!(2, writer1.get_matched_subscriptions().unwrap().len());
            assert_eq!(2, writer2.get_matched_subscriptions().unwrap().len());
            assert_eq!(2, reader1.get_matched_publications().unwrap().len());
            assert_eq!(2, reader2.get_matched_publications().unwrap().len());
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
            vec![unicast_address.into()],
            multicast_address.into(),
        )
        .unwrap();
        let participant1 = DdsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications1.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            communications1.metatraffic_unicast_locator_list(),
            communications1.metatraffic_multicast_locator_list(),
            communications1.default_unicast_locator_list(),
            vec![],
        ));
        create_builtins(participant1.clone()).unwrap();
        let participant1_proxy = DomainParticipantProxy::new(participant1.downgrade());

        let mut communications2 = Communications::find_available(
            domain_id,
            [0; 6],
            vec![unicast_address.into()],
            multicast_address.into(),
        )
        .unwrap();
        let participant2 = DdsShared::new(DomainParticipantAttributes::<RtpsStructureImpl>::new(
            communications2.guid_prefix,
            domain_id,
            "".to_string(),
            DomainParticipantQos::default(),
            communications2.metatraffic_unicast_locator_list(),
            communications2.metatraffic_multicast_locator_list(),
            communications2.default_unicast_locator_list(),
            vec![],
        ));
        create_builtins(participant2.clone()).unwrap();
        let participant2_proxy = DomainParticipantProxy::new(participant2.downgrade());

        // Match SEDP endpoints
        {
            task_announce_participant(participant1.clone()).unwrap();
            task_announce_participant(participant2.clone()).unwrap();

            communications1.metatraffic_unicast.send_publisher_message(
                participant1.builtin_publisher.read_lock().as_ref().unwrap(),
            );
            communications1.metatraffic_unicast.send_publisher_message(
                participant2.builtin_publisher.read_lock().as_ref().unwrap(),
            );

            communications1.metatraffic_multicast.receive(
                &[],
                core::slice::from_ref(
                    participant1
                        .builtin_subscriber
                        .read_lock()
                        .as_ref()
                        .unwrap(),
                ),
            );
            communications2.metatraffic_multicast.receive(
                &[],
                core::slice::from_ref(
                    participant2
                        .builtin_subscriber
                        .read_lock()
                        .as_ref()
                        .unwrap(),
                ),
            );

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
            communications1.metatraffic_unicast.send_publisher_message(
                participant1.builtin_publisher.read_lock().as_ref().unwrap(),
            );
            communications2.metatraffic_unicast.send_publisher_message(
                participant2.builtin_publisher.read_lock().as_ref().unwrap(),
            );

            communications1.metatraffic_unicast.receive(
                &[],
                core::slice::from_ref(
                    participant1
                        .builtin_subscriber
                        .read_lock()
                        .as_ref()
                        .unwrap(),
                ),
            );
            communications2.metatraffic_unicast.receive(
                &[],
                core::slice::from_ref(
                    participant2
                        .builtin_subscriber
                        .read_lock()
                        .as_ref()
                        .unwrap(),
                ),
            );
        }

        // Before the SEDP task the readers/writers are not matched
        {
            assert_eq!(0, writer1.get_matched_subscriptions().unwrap().len());
            assert_eq!(0, writer2.get_matched_subscriptions().unwrap().len());
            assert_eq!(0, reader1.get_matched_publications().unwrap().len());
            assert_eq!(0, reader2.get_matched_publications().unwrap().len());
        }

        // call SEDP task
        task_sedp_reader_discovery(participant1.clone()).unwrap();
        task_sedp_reader_discovery(participant2.clone()).unwrap();
        task_sedp_writer_discovery(participant1.clone()).unwrap();
        task_sedp_writer_discovery(participant2.clone()).unwrap();

        // After the SEDP task the readers/writers are matched only on the same participant
        {
            assert_eq!(1, writer1.get_matched_subscriptions().unwrap().len());
            assert_eq!(1, writer2.get_matched_subscriptions().unwrap().len());
            assert_eq!(1, reader1.get_matched_publications().unwrap().len());
            assert_eq!(1, reader2.get_matched_publications().unwrap().len());
        }
    }
}
