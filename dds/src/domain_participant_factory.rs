use std::{
    net::{Ipv4Addr, UdpSocket},
    str::FromStr,
};

use rust_dds_api::{
    builtin_topics::ParticipantBuiltinTopicData,
    dcps_psm::{BuiltInTopicKey, DomainId, StatusMask},
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::{
        qos::{DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos},
        qos_policy::UserDataQosPolicy,
    },
    publication::data_writer::DataWriter,
};
use rust_dds_rtps_implementation::{
    data_representation_builtin_endpoints::spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    dds_impl::{
        data_writer_impl::DataWriterImpl, domain_participant_impl::DomainParticipantImpl,
        publisher_impl::PublisherImpl, subscriber_impl::SubscriberImpl,
    },
    rtps_impl::{rtps_group_impl::RtpsGroupImpl, rtps_writer_impl::RtpsWriterImpl},
    utils::shared_object::RtpsShared,
};
use rust_rtps_pim::{
    behavior::types::Duration,
    discovery::{
        spdp::{
            builtin_endpoints::SpdpBuiltinParticipantWriter, participant_proxy::ParticipantProxy,
        },
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    messages::types::Count,
    structure::types::{
        EntityId, Guid, GuidPrefix, LOCATOR_KIND_UDPv4, Locator, BUILT_IN_READER_GROUP,
        BUILT_IN_WRITER_GROUP, LOCATOR_INVALID, PROTOCOLVERSION, VENDOR_ID_S2E,
    },
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
        domain_id: DomainId,
        qos: Option<DomainParticipantQos>,
        _a_listener: Option<Box<dyn DomainParticipantListener>>,
        _mask: StatusMask,
    ) -> Option<DomainParticipantImpl> {
        let guid_prefix = GuidPrefix([3; 12]);

        let socket = UdpSocket::bind("127.0.0.1:7400").unwrap();
        socket.set_nonblocking(true).unwrap();
        socket
            .join_multicast_v4(
                &Ipv4Addr::from_str("239.255.0.1").unwrap(),
                &Ipv4Addr::from_str("127.0.0.1").unwrap(),
            )
            .unwrap();
        let metatraffic_transport = Box::new(UdpTransport::new(socket));

        let socket = UdpSocket::bind("127.0.0.1:7410").unwrap();
        socket.set_nonblocking(true).unwrap();
        let default_transport = Box::new(UdpTransport::new(socket));

        let spdp_builtin_participant_writer_qos = DataWriterQos::default();
        let spdp_discovery_locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            7400,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
        );
        let spdp_builtin_participant_rtps_writer: RtpsWriterImpl =
            SpdpBuiltinParticipantWriter::create(guid_prefix, &[], &[], &[spdp_discovery_locator]);

        let dds_participant_data = ParticipantBuiltinTopicData {
            key: BuiltInTopicKey { value: [0; 3] },
            user_data: UserDataQosPolicy { value: &[1, 2, 3] },
        };
        let participant_proxy = ParticipantProxy {
            domain_id: domain_id as u32,
            domain_tag: "ab",
            protocol_version: PROTOCOLVERSION,
            guid_prefix,
            vendor_id: VENDOR_ID_S2E,
            expects_inline_qos: false,
            metatraffic_unicast_locator_list: vec![LOCATOR_INVALID, LOCATOR_INVALID],
            metatraffic_multicast_locator_list: vec![],
            default_unicast_locator_list: vec![],
            default_multicast_locator_list: vec![],
            available_builtin_endpoints: BuiltinEndpointSet::new(
                BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER
                    | BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR,
            ),
            manual_liveliness_count: Count(0),
            builtin_endpoint_qos: BuiltinEndpointQos::default(),
        };
        let lease_duration = Duration {
            seconds: 100,
            fraction: 0,
        };

        let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
            dds_participant_data,
            participant_proxy,
            lease_duration,
        };

        let spdp_builtin_participant_writer = RtpsShared::new(DataWriterImpl::new(
            spdp_builtin_participant_writer_qos,
            spdp_builtin_participant_rtps_writer,
        ));

        spdp_builtin_participant_writer
            .write()
            .write_w_timestamp(
                spdp_discovered_participant_data,
                None,
                rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
            )
            .unwrap();

        let builtin_publisher_storage = RtpsShared::new(PublisherImpl::new(
            PublisherQos::default(),
            RtpsGroupImpl::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_WRITER_GROUP),
            )),
            vec![spdp_builtin_participant_writer],
        ));
        let builtin_subscriber_storage = RtpsShared::new(SubscriberImpl::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], BUILT_IN_READER_GROUP),
            )),
            Vec::new(),
        ));

        let domain_participant = DomainParticipantImpl::new(
            guid_prefix,
            qos.unwrap_or_default(),
            builtin_subscriber_storage,
            builtin_publisher_storage,
            metatraffic_transport,
            default_transport,
        );

        Some(domain_participant)
    }
}

// impl rust_dds_rtps_implementation::dds_type::DDSType for SPDPdiscoveredParticipantDataUdp {
//     fn type_name() -> &'static str {
//         todo!()
//     }

//     fn has_key() -> bool {
//         todo!()
//     }
// }
