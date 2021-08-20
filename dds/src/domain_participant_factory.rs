use std::{
    net::{Ipv4Addr, UdpSocket},
    str::FromStr,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use rust_dds_api::{
    dcps_psm::{DomainId, StatusMask},
    domain::{
        domain_participant::DomainParticipant,
        domain_participant_listener::DomainParticipantListener,
    },
    infrastructure::qos::{DataWriterQos, DomainParticipantQos, PublisherQos, SubscriberQos},
};
use rust_dds_rtps_implementation::{
    dds_impl::{
        data_writer_impl::DataWriterStorage,
        domain_participant_impl::{DomainParticipantImpl, DomainParticipantStorage},
        publisher_impl::PublisherStorage,
        subscriber_impl::SubscriberStorage,
    },
    rtps_impl::{
        rtps_group_impl::RtpsGroupImpl, rtps_participant_impl::RtpsParticipantImpl,
        rtps_writer_impl::RtpsWriterImpl,
    },
    utils::shared_object::RtpsShared,
};
use rust_rtps_pim::{
    behavior::{
        types::Duration,
        writer::writer::{RtpsWriter, RtpsWriterOperations},
    },
    discovery::{
        spdp::builtin_endpoints::SpdpBuiltinParticipantWriter,
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    messages::types::Count,
    structure::{
        types::{ChangeKind, EntityId, EntityKind, Guid, LOCATOR_KIND_UDPv4, Locator},
        RtpsEntity, RtpsHistoryCache, RtpsParticipant,
    },
};
use rust_rtps_udp_psm::{
    builtin_endpoints::data::SPDPdiscoveredParticipantDataUdp, serialize::to_bytes_le,
};

use crate::udp_transport::{receive_udp_data, send_udp_data, UdpTransport};

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
    pub fn create_participant(
        domain_id: DomainId,
        qos: Option<DomainParticipantQos>,
        _a_listener: Option<Box<dyn DomainParticipantListener>>,
        _mask: StatusMask,
    ) -> Option<impl DomainParticipant> {
        let guid_prefix = [3; 12];

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

        let rtps_participant = RtpsParticipantImpl::new(guid_prefix);

        // let spdp_builtin_participant_reader: RTPSReaderImpl =
        //     SpdpBuiltinParticipantReader::create(guid_prefix);

        // rtps_participant
        //     .builtin_reader_group
        //     .lock()
        //     .add_reader(RtpsShared::new(spdp_builtin_participant_reader));

        let spdp_builtin_participant_writer_qos = DataWriterQos::default();
        let spdp_discovery_locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            7400,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
        );
        let mut spdp_builtin_participant_rtps_writer: RtpsWriterImpl =
            SpdpBuiltinParticipantWriter::create(guid_prefix, &[], &[], &[spdp_discovery_locator]);

        let lease_duration = Duration {
            seconds: 30,
            fraction: 0,
        };
        let spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::new(
            &(domain_id as u32),
            &"abc",
            rtps_participant.protocol_version(),
            rtps_participant.guid(),
            rtps_participant.vendor_id(),
            &false,
            &[],
            &[spdp_discovery_locator],
            &[],
            &[],
            &BuiltinEndpointSet::new(
                BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER
                    | BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR,
            ),
            &Count(0),
            &BuiltinEndpointQos::default(),
            &lease_duration,
        );
        let spdp_discovered_participant_data_bytes =
            to_bytes_le(&spdp_discovered_participant_data).unwrap();
        let cc = spdp_builtin_participant_rtps_writer.new_change(
            ChangeKind::Alive,
            &spdp_discovered_participant_data_bytes,
            &[],
            1,
        );

        spdp_builtin_participant_rtps_writer
            .writer_cache_mut()
            .add_change(&cc);

        let spdp_builtin_participant_writer = RtpsShared::new(DataWriterStorage::new(
            spdp_builtin_participant_writer_qos,
            spdp_builtin_participant_rtps_writer,
        ));

        let builtin_publisher_storage = vec![RtpsShared::new(PublisherStorage::new(
            PublisherQos::default(),
            RtpsGroupImpl::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], EntityKind::BuiltInWriterGroup),
            )),
            vec![spdp_builtin_participant_writer],
        ))];
        let builtin_subscriber_storage = vec![RtpsShared::new(SubscriberStorage::new(
            SubscriberQos::default(),
            RtpsGroupImpl::new(Guid::new(
                guid_prefix,
                EntityId::new([0, 0, 0], EntityKind::BuiltInReaderGroup),
            )),
            Vec::new(),
        ))];

        let domain_participant_storage = RtpsShared::new(DomainParticipantStorage::new(
            qos.unwrap_or_default(),
            rtps_participant,
            builtin_subscriber_storage,
            builtin_publisher_storage,
            metatraffic_transport,
            default_transport,
        ));
        let is_enabled = Arc::new(AtomicBool::new(false));
        let is_enabled_cloned = is_enabled.clone();
        let domain_participant_storage_cloned = domain_participant_storage.clone();
        let communication_thread_handle = std::thread::spawn(move || loop {
            // if is_enabled_cloned.load(atomic::Ordering::Relaxed) {
            //     let domain_participant_storage_lock = domain_participant_storage_cloned.lock();
            //     send_udp_data(
            //         domain_participant_storage_lock.rtps_participant(),
            //         domain_participant_storage_lock.builtin_publisher_storage(),
            //         &mut metatraffic_transport,
            //     );
            //     receive_udp_data(
            //         domain_participant_storage_lock.rtps_participant(),
            //         domain_participant_storage_lock.builtin_subscriber_storage(),
            //         &mut metatraffic_transport,
            //     );

            //     send_udp_data(
            //         domain_participant_storage_lock.rtps_participant(),
            //         domain_participant_storage_lock.user_defined_publisher_storage(),
            //         &mut default_transport,
            //     );
            //     receive_udp_data(
            //         domain_participant_storage_lock.rtps_participant(),
            //         domain_participant_storage_lock.user_defined_subscriber_storage(),
            //         &mut default_transport,
            //     );
            //     std::thread::sleep(std::time::Duration::from_millis(100));
            // }
        });

        let worker_threads = vec![communication_thread_handle];

        let domain_participant =
            DomainParticipantImpl::new(is_enabled, domain_participant_storage, worker_threads);

        Some(domain_participant)
    }
}
