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
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::qos::DomainParticipantQos,
};
use rust_dds_rtps_implementation::{
    dds_impl::domain_participant_impl::DomainParticipantImpl,
    rtps_impl::{
        rtps_participant_impl::RTPSParticipantImpl, rtps_reader_impl::RTPSReaderImpl,
        rtps_writer_impl::RTPSWriterImpl,
    },
    utils::{
        message_receiver::MessageReceiver, message_sender::send_data, shared_object::RtpsShared,
        transport::TransportRead,
    },
};
use rust_rtps_pim::{
    behavior::{
        reader::reader::RTPSReader,
        types::Duration,
        writer::writer::{RTPSWriter, RTPSWriterOperations},
    },
    discovery::{
        participant_discovery::ParticipantDiscovery,
        spdp::builtin_endpoints::{SpdpBuiltinParticipantReader, SpdpBuiltinParticipantWriter},
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    messages::types::Count,
    structure::{
        types::{ChangeKind, LOCATOR_KIND_UDPv4, Locator},
        RTPSEntity, RTPSHistoryCache, RTPSParticipant,
    },
};
use rust_rtps_udp_psm::builtin_endpoints::data::SPDPdiscoveredParticipantDataUdp;

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
    pub fn create_participant(
        domain_id: DomainId,
        _qos: Option<DomainParticipantQos>,
        _a_listener: Option<Box<dyn DomainParticipantListener>>,
        _mask: StatusMask,
    ) -> Option<DomainParticipantImpl> {
        let guid_prefix = [3; 12];

        let socket = UdpSocket::bind("127.0.0.1:7400").unwrap();
        socket.set_nonblocking(true).unwrap();
        socket
            .join_multicast_v4(
                &Ipv4Addr::from_str("239.255.0.1").unwrap(),
                &Ipv4Addr::from_str("127.0.0.1").unwrap(),
            )
            .unwrap();
        let mut transport = UdpTransport::new(socket);

        let rtps_participant = RTPSParticipantImpl::new(guid_prefix);
        let is_enabled = Arc::new(AtomicBool::new(false));
        let is_enabled_thread = is_enabled.clone();

        let spdp_discovery_locator = Locator::new(
            LOCATOR_KIND_UDPv4,
            7400,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
        );
        let mut spdp_builtin_participant_writer: RTPSWriterImpl =
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
        let spdp_discovered_participant_data_bytes = spdp_discovered_participant_data.to_bytes().unwrap();
        let cc = spdp_builtin_participant_writer.new_change(
            ChangeKind::Alive,
            &spdp_discovered_participant_data_bytes,
            &[],
            1,
        );

        spdp_builtin_participant_writer
            .writer_cache_mut()
            .add_change(&cc);

        let spdp_builtin_participant_reader: RTPSReaderImpl =
            SpdpBuiltinParticipantReader::create(guid_prefix);

        rtps_participant
            .builtin_reader_group
            .lock()
            .add_reader(RtpsShared::new(spdp_builtin_participant_reader));

        let rtps_participant_impl = RtpsShared::new(rtps_participant);
        let rtps_participant_shared = rtps_participant_impl.clone();

        std::thread::spawn(move || loop {
            if is_enabled_thread.load(atomic::Ordering::Relaxed) {
                if let Some(mut rtps_participant) = rtps_participant_shared.try_lock() {
                    if let Some((source_locator, message)) = transport.read() {
                        MessageReceiver::new().process_message(
                            guid_prefix,
                            &*rtps_participant.builtin_reader_group.lock(),
                            source_locator,
                            &message,
                        );
                    }
                    send_data(
                        &*rtps_participant,
                        &mut spdp_builtin_participant_writer,
                        &mut transport,
                    );
                    let mut spdp_discovered_participant_datas =
                        Vec::<SPDPdiscoveredParticipantDataUdp>::new();
                    {
                        let builtin_reader_group = rtps_participant.builtin_reader_group.lock();
                        let spdp_builtin_participant_reader =
                            builtin_reader_group.reader_list()[0].lock();
                        if let Some(seq_num_min) = spdp_builtin_participant_reader
                            .reader_cache()
                            .get_seq_num_min()
                        {
                            let seq_num_max = spdp_builtin_participant_reader
                                .reader_cache()
                                .get_seq_num_max()
                                .unwrap();
                            for seq_num in seq_num_min..seq_num_max {
                                if let Some(change) = spdp_builtin_participant_reader
                                    .reader_cache()
                                    .get_change(&seq_num)
                                {
                                    if let Ok(spdp_discovered_participant_data) =
                                        SPDPdiscoveredParticipantDataUdp::from_bytes(
                                            change.data_value(),
                                        )
                                    {
                                        spdp_discovered_participant_datas
                                            .push(spdp_discovered_participant_data);
                                    }
                                }
                            }
                        }
                    }

                    for spdp_discovered_participant_data in spdp_discovered_participant_datas {
                        rtps_participant
                            .discovered_participant_add(&spdp_discovered_participant_data);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        let domain_participant = DomainParticipantImpl::new(rtps_participant_impl, is_enabled);

        Some(domain_participant)
    }
}
