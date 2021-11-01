use std::{
    net::{Ipv4Addr, UdpSocket},
    str::FromStr,
    sync::mpsc::sync_channel,
};

use rust_dds_api::{
    dcps_psm::{DomainId, StatusMask},
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::qos::{
        DataReaderQos, DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos,
    },
};
use rust_dds_rtps_implementation::{
    data_representation_builtin_endpoints::{
        sedp_discovered_reader_data::SedpDiscoveredReaderData,
        sedp_discovered_topic_data::SedpDiscoveredTopicData,
        sedp_discovered_writer_data::SedpDiscoveredWriterData,
        spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    },
    dds_impl::{
        data_reader_impl::{DataReaderImpl, RtpsReaderFlavor},
        data_writer_impl::{DataWriterImpl, RtpsWriterFlavor},
        domain_participant_impl::DomainParticipantImpl,
        publisher_impl::PublisherImpl,
        subscriber_impl::SubscriberImpl,
    },
    utils::shared_object::rtps_shared_new,
};
use rust_rtps_pim::{
    behavior::writer::{
        reader_locator::RtpsReaderLocator, stateless_writer::RtpsStatelessWriterOperations,
    },
    structure::{
        types::{
            EntityId, Guid, GuidPrefix, LOCATOR_KIND_UDPv4, Locator, BUILT_IN_READER_GROUP,
            BUILT_IN_WRITER_GROUP,
        },
        RtpsGroup,
    },
};
use rust_rtps_psm::discovery::{
    sedp::builtin_endpoints::{
        SedpBuiltinPublicationsReader, SedpBuiltinPublicationsWriter,
        SedpBuiltinSubscriptionsReader, SedpBuiltinSubscriptionsWriter, SedpBuiltinTopicsReader,
        SedpBuiltinTopicsWriter,
    },
    spdp::builtin_endpoints::{SpdpBuiltinParticipantReader, SpdpBuiltinParticipantWriter},
};

use crate::udp_transport::UdpTransport;

/// The DomainParticipant object plays several roles:
/// - It acts as a container for all other Entity objects.
/// - It acts as factory for the Publisher, Subscriber, Topic, and MultiTopic Entity objects.
/// - It represents the participation of the application on a communication plane that isolates applications running on the
/// same set of physical computers from each other. A domain establishes a “virtual network” linking all applications that
/// share the same domainId and isolating them from applications running on different domains. In this way, several
/// independent distributed applications can coexist in the same physical network without interfering, or even being aware
/// of each other.
/// - It provides administration services in the domain, offering operations that allow the application to ‘ignore’ locally any
/// information about a given participant (ignore_participant), publication (ignore_publication), subscription
/// (ignore_subscription), or topic (ignore_topic).
///
/// The following sub clauses explain all the operations in detail.
/// The following operations may be called even if the DomainParticipant is not enabled. Other operations will have the value
/// NOT_ENABLED if called on a disabled DomainParticipant:
/// - Operations defined at the base-class level namely, set_qos, get_qos, set_listener, get_listener, and enable.
/// - Factory methods: create_topic, create_publisher, create_subscriber, delete_topic, delete_publisher,
/// delete_subscriber
/// - Operations that access the status: get_statuscondition
pub struct DomainParticipantFactory;

impl DomainParticipantFactory {
    /// This operation creates a new DomainParticipant object. The DomainParticipant signifies that the calling application intends
    /// to join the Domain identified by the domain_id argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no DomainParticipant will be created.
    /// The special value PARTICIPANT_QOS_DEFAULT can be used to indicate that the DomainParticipant should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation get_default_participant_qos (2.2.2.2.2.6) and using the resulting
    /// QoS to create the DomainParticipant.
    /// In case of failure, the operation will return a ‘nil’ value (as specified by the platform).
    ///
    /// Developer note: Ideally this method should return impl DomainParticipant. However because of the GAT workaround used there is no way
    /// to call,e.g. create_topic(), because we can't write impl DomainParticipant + for<'t, T> TopicGAT<'t, T> on the return. This issue will
    /// probably be solved once the GAT functionality is available on stable.
    pub fn create_participant(
        _domain_id: DomainId,
        qos: Option<DomainParticipantQos>,
        _a_listener: Option<Box<dyn DomainParticipantListener>>,
        _mask: StatusMask,
    ) -> Option<DomainParticipantImpl> {
        // /////// Define guid prefix
        let guid_prefix = GuidPrefix([3; 12]);

        // /////// Create transports
        let socket = UdpSocket::bind("127.0.0.1:7400").unwrap();
        socket.set_nonblocking(true).unwrap();
        socket
            .join_multicast_v4(
                &Ipv4Addr::from_str("239.255.0.1").unwrap(),
                &Ipv4Addr::from_str("127.0.0.1").unwrap(),
            )
            .unwrap();
        socket.set_multicast_loop_v4(true).unwrap();
        let metatraffic_transport = Box::new(UdpTransport::new(socket));

        let socket = UdpSocket::bind("127.0.0.1:7410").unwrap();
        socket.set_nonblocking(true).unwrap();
        let default_transport = Box::new(UdpTransport::new(socket));

        let (metatraffic_locator_message_sender, metatraffic_locator_message_receiver) =
            sync_channel(17);
        let (metatraffic_locator_list_message_sender, metatraffic_locator_list_message_receiver) =
            sync_channel(17);

        // /////// Create SPDP and SEDP endpoints
        let spdp_builtin_participant_rtps_reader =
            SpdpBuiltinParticipantReader::create(guid_prefix, vec![], vec![]);
        let mut spdp_builtin_participant_rtps_writer =
            SpdpBuiltinParticipantWriter::create(guid_prefix, vec![], vec![]);
        let sedp_builtin_publications_rtps_reader =
            SedpBuiltinPublicationsReader::create(guid_prefix, vec![], vec![]);
        let sedp_builtin_publications_rtps_writer =
            SedpBuiltinPublicationsWriter::create(guid_prefix, vec![], vec![]);
        let sedp_builtin_subscriptions_rtps_reader =
            SedpBuiltinSubscriptionsReader::create(guid_prefix, vec![], vec![]);
        let sedp_builtin_subscriptions_rtps_writer =
            SedpBuiltinSubscriptionsWriter::create(guid_prefix, vec![], vec![]);
        let sedp_builtin_topics_rtps_reader =
            SedpBuiltinTopicsReader::create(guid_prefix, vec![], vec![]);
        let sedp_builtin_topics_rtps_writer =
            SedpBuiltinTopicsWriter::create(guid_prefix, vec![], vec![]);

        // ////////// Configure SPDP reader locator
        let spdp_discovery_locator = RtpsReaderLocator::new(
            Locator::new(
                LOCATOR_KIND_UDPv4,
                7400,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
            ),
            false,
        );
        spdp_builtin_participant_rtps_writer.reader_locator_add(spdp_discovery_locator);

        // ///////// Create built-in DDS data readers and data writers
        let spdp_builtin_participant_dds_data_reader =
            rtps_shared_new(DataReaderImpl::<SpdpDiscoveredParticipantData>::new(
                DataReaderQos::default(),
                RtpsReaderFlavor::Stateless(spdp_builtin_participant_rtps_reader),
            ));

        let spdp_builtin_participant_dds_data_writer = rtps_shared_new(DataWriterImpl::new(
            DataWriterQos::default(),
            RtpsWriterFlavor::Stateless(spdp_builtin_participant_rtps_writer),
            metatraffic_locator_message_sender.clone(),
            metatraffic_locator_list_message_sender.clone(),
        ));

        let sedp_builtin_publications_dds_data_reader =
            rtps_shared_new(DataReaderImpl::<SedpDiscoveredWriterData>::new(
                DataReaderQos::default(),
                RtpsReaderFlavor::Stateful(sedp_builtin_publications_rtps_reader),
            ));

        let sedp_builtin_publications_dds_data_writer = rtps_shared_new(DataWriterImpl::new(
            DataWriterQos::default(),
            RtpsWriterFlavor::new_stateful(sedp_builtin_publications_rtps_writer),
            metatraffic_locator_message_sender.clone(),
            metatraffic_locator_list_message_sender.clone(),
        ));

        let sedp_builtin_subscriptions_dds_data_reader =
            rtps_shared_new(DataReaderImpl::<SedpDiscoveredReaderData>::new(
                DataReaderQos::default(),
                RtpsReaderFlavor::Stateful(sedp_builtin_subscriptions_rtps_reader),
            ));

        let sedp_builtin_subscriptions_dds_data_writer = rtps_shared_new(DataWriterImpl::new(
            DataWriterQos::default(),
            RtpsWriterFlavor::new_stateful(sedp_builtin_subscriptions_rtps_writer),
            metatraffic_locator_message_sender.clone(),
            metatraffic_locator_list_message_sender.clone(),
        ));

        let sedp_builtin_topics_dds_data_reader =
            rtps_shared_new(DataReaderImpl::<SedpDiscoveredTopicData>::new(
                DataReaderQos::default(),
                RtpsReaderFlavor::Stateful(sedp_builtin_topics_rtps_reader),
            ));

        let sedp_builtin_topics_dds_data_writer = rtps_shared_new(DataWriterImpl::new(
            DataWriterQos::default(),
            RtpsWriterFlavor::new_stateful(sedp_builtin_topics_rtps_writer),
            metatraffic_locator_message_sender.clone(),
            metatraffic_locator_list_message_sender.clone(),
        ));

        // ////// Create built-in publisher and subscriber
        let builtin_publisher_storage = rtps_shared_new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
            )),
            vec![
                spdp_builtin_participant_dds_data_writer,
                sedp_builtin_publications_dds_data_writer,
                sedp_builtin_subscriptions_dds_data_writer,
                sedp_builtin_topics_dds_data_writer,
            ],
            metatraffic_locator_message_sender.clone(),
            metatraffic_locator_list_message_sender.clone(),
        ));
        let builtin_subscriber_storage = rtps_shared_new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroup::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
            )),
            vec![
                spdp_builtin_participant_dds_data_reader,
                sedp_builtin_publications_dds_data_reader,
                sedp_builtin_subscriptions_dds_data_reader,
                sedp_builtin_topics_dds_data_reader,
            ],
        ));

        let domain_participant = DomainParticipantImpl::new(
            guid_prefix,
            qos.unwrap_or_default(),
            builtin_subscriber_storage,
            builtin_publisher_storage,
            metatraffic_transport,
            default_transport,
            metatraffic_locator_message_sender,
            metatraffic_locator_message_receiver,
            metatraffic_locator_list_message_sender,
            metatraffic_locator_list_message_receiver,
        );

        Some(domain_participant)
    }
}

// let dds_participant_data = ParticipantBuiltinTopicData {
//     key: BuiltInTopicKey { value: [0; 3] },
//     user_data: UserDataQosPolicy {
//         value: vec![1, 2, 3],
//     },
// };
// let participant_proxy = ParticipantProxy {
//     domain_id: domain_id as u32,
//     domain_tag: "ab",
//     protocol_version: PROTOCOLVERSION,
//     guid_prefix,
//     vendor_id: VENDOR_ID_S2E,
//     expects_inline_qos: false,
//     metatraffic_unicast_locator_list: vec![LOCATOR_INVALID, LOCATOR_INVALID],
//     metatraffic_multicast_locator_list: vec![],
//     default_unicast_locator_list: vec![],
//     default_multicast_locator_list: vec![],
//     available_builtin_endpoints: BuiltinEndpointSet::new(
//         BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER
//             | BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR,
//     ),
//     manual_liveliness_count: Count(0),
//     builtin_endpoint_qos: BuiltinEndpointQos::default(),
// };
// let lease_duration = Duration {
//     seconds: 100,
//     fraction: 0,
// };

// let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
//     dds_participant_data,
//     participant_proxy,
//     lease_duration,
// };

// rtps_shared_write_lock(&spdp_builtin_participant_writer)
//     .write_w_timestamp(
//         spdp_discovered_participant_data,
//         None,
//         rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
//     )
//     .unwrap();
