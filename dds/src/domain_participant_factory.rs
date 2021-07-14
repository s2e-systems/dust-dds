use std::{
    net::UdpSocket,
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
        rtps_participant_impl::RTPSParticipantImpl,
        rtps_reader_locator_impl::RTPSReaderLocatorImpl, rtps_writer_impl::RTPSWriterImpl,
    },
    utils::{
        message_receiver::MessageReceiver,
        message_sender::create_submessages,
        shared_object::RtpsShared,
        transport::{TransportRead, TransportWrite},
    },
};
use rust_rtps_pim::{
    behavior::{
        types::Duration,
        writer::{
            reader_locator::RTPSReaderLocatorOperations,
            stateless_writer::RTPSStatelessWriterOperations,
            writer::{RTPSWriter, RTPSWriterOperations},
        },
    },
    discovery::ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER,
    messages::{RTPSMessage, RtpsMessageHeader},
    structure::{
        types::{ChangeKind, LOCATOR_KIND_UDPv4, Locator, GUID},
        RTPSEntity, RTPSHistoryCache, RTPSParticipant,
    },
};
use rust_rtps_udp_psm::{message::RTPSMessageUdp, psm::RtpsUdpPsm};

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
        _domain_id: DomainId,
        _qos: Option<DomainParticipantQos>,
        _a_listener: Option<Box<dyn DomainParticipantListener>>,
        _mask: StatusMask,
    ) -> Option<DomainParticipantImpl> {
        // let interface = "Wi-Fi";
        // let unicast_locator = Locator::new(0, 7400, [1; 16]);
        // let multicast_locator = Locator::new(0, 7400, [2; 16]);
        // let userdata_transport =
        //     Box::new(MemoryTransport::new(unicast_locator, vec![multicast_locator]).unwrap());
        // let metatraffic_transport =
        //     Box::new(MemoryTransport::new(unicast_locator, vec![multicast_locator]).unwrap());
        // let qos = qos.unwrap_or_default();

        // let configuration = DomainParticipantImplConfiguration {
        //     userdata_transport,
        //     metatraffic_transport,
        //     domain_tag: "",
        //     lease_duration: Duration {
        //         seconds: 30,
        //         fraction: 0,
        //     },
        //     spdp_locator_list: vec![Locator::new_udpv4(7400, [239, 255, 0, 0])],
        // };
        let guid_prefix = [3; 12];

        let socket = UdpSocket::bind("127.0.0.1:7400").unwrap();
        let mut transport = UdpTransport::new(socket);

        let rtps_participant = RTPSParticipantImpl::new(guid_prefix);
        let is_enabled = Arc::new(AtomicBool::new(false));
        let is_enabled_thread = is_enabled.clone();

        let spdp_discovery_writer_guid =
            GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER);
        let mut spdp_discovery_writer = <RTPSWriterImpl as RTPSStatelessWriterOperations>::new(
            spdp_discovery_writer_guid,
            rust_rtps_pim::structure::types::TopicKind::WithKey,
            rust_rtps_pim::structure::types::ReliabilityKind::BestEffort,
            &[],
            &[],
            true,
            Duration(0),
            Duration(0),
            Duration(0),
            None,
        );
        let spdp_discovery_writer_locator = RTPSReaderLocatorImpl::new(
            Locator::new(
                LOCATOR_KIND_UDPv4,
                7400,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
            ),
            false,
        );

        spdp_discovery_writer.reader_locator_add(spdp_discovery_writer_locator);

        let cc =
            spdp_discovery_writer.new_change(ChangeKind::Alive, vec![1, 2, 3, 4, 5, 6, 7], (), 1);
        spdp_discovery_writer.writer_cache_mut().add_change(cc);

        rtps_participant
            .builtin_writer_group
            .lock()
            .add_writer(RtpsShared::new(spdp_discovery_writer));

        let rtps_participant_impl = RtpsShared::new(rtps_participant);
        let rtps_participant_shared = rtps_participant_impl.clone();

        std::thread::spawn(move || loop {
            if is_enabled_thread.load(atomic::Ordering::Relaxed) {
                if let Some(rtps_participant) = rtps_participant_shared.try_lock() {
                    let protocol_version = rtps_participant.protocol_version();
                    let vendor_id = rtps_participant.vendor_id();
                    let header = RtpsMessageHeader {
                        protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
                        version: *protocol_version,
                        vendor_id: *vendor_id,
                        guid_prefix: *rtps_participant.guid().prefix(),
                    };

                    if let Some((source_locator, message)) = transport.read() {
                        MessageReceiver::new().process_message(
                            guid_prefix,
                            &*rtps_participant.builtin_reader_group.lock(),
                            source_locator,
                            &message,
                        );
                    }

                    // for writer_group in rtps_participant.writer_groups() {
                    let writer_group = rtps_participant.builtin_writer_group.lock();
                    for writer in writer_group.writer_list() {
                        let mut writer = writer.lock();
                        writer.unsent_changes_reset();
                        let destined_submessages =
                            create_submessages::<RtpsUdpPsm, _>(&mut *writer);
                        for (dst_locator, submessages) in destined_submessages {
                            let message = RTPSMessageUdp::new(&header, submessages);
                            transport.write(&message, &dst_locator);
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
                // }
            }
        });

        let domain_participant = DomainParticipantImpl::new(rtps_participant_impl, is_enabled);
        // let participant = DomainParticipant::Rtps(domain_participant);

        // let domain_participant_impl =
        // DomainParticipantImpl::new(guid_prefix.into()); // domain_id, qos.clone(), a_listener, mask, configuration
        // let participant = DomainParticipant::new(domain_participant_impl);

        // if enabled {
        //     new_participant.enable().ok()?;
        // }

        Some(domain_participant)
    }

    // pub fn delete_participant(_a_participant: impl DomainParticipant) {}
}
