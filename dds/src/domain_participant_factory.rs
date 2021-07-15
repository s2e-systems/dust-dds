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
        rtps_participant_impl::RTPSParticipantImpl,
        rtps_reader_locator_impl::RTPSReaderLocatorImpl, rtps_writer_impl::RTPSWriterImpl,
    },
    utils::{
        message_receiver::MessageReceiver, message_sender::send_data, shared_object::RtpsShared,
        transport::TransportRead,
    },
};
use rust_rtps_pim::{
    behavior::writer::{
        reader_locator::RTPSReaderLocatorOperations,
        stateless_writer::RTPSStatelessWriterOperations,
        writer::{RTPSWriter, RTPSWriterOperations},
    },
    discovery::spdp::builtin_endpoints::SpdpBuiltinParticipantWriter,
    structure::{
        types::{ChangeKind, LOCATOR_KIND_UDPv4, Locator},
        RTPSEntity, RTPSHistoryCache, RTPSParticipant,
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
    pub fn create_participant(
        _domain_id: DomainId,
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

        let mut spdp_builtin_participant_writer =
            SpdpBuiltinParticipantWriter::create::<RTPSWriterImpl>(guid_prefix);
        let spdp_discovery_writer_locator = RTPSReaderLocatorImpl::new(
            Locator::new(
                LOCATOR_KIND_UDPv4,
                7400,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1],
            ),
            false,
        );

        spdp_builtin_participant_writer.reader_locator_add(spdp_discovery_writer_locator);

        let cc = spdp_builtin_participant_writer.new_change(
            ChangeKind::Alive,
            vec![0, 0, 0, 0, 5, 6, 7],
            (),
            1,
        );
        spdp_builtin_participant_writer
            .writer_cache_mut()
            .add_change(cc);

        rtps_participant
            .builtin_writer_group
            .lock()
            .add_writer(RtpsShared::new(spdp_builtin_participant_writer));

        let rtps_participant_impl = RtpsShared::new(rtps_participant);
        let rtps_participant_shared = rtps_participant_impl.clone();

        std::thread::spawn(move || loop {
            if is_enabled_thread.load(atomic::Ordering::Relaxed) {
                if let Some(rtps_participant) = rtps_participant_shared.try_lock() {
                    let protocol_version = *rtps_participant.protocol_version();
                    let vendor_id = *rtps_participant.vendor_id();
                    let guid_prefix = *rtps_participant.guid().prefix();

                    if let Some((source_locator, message)) = transport.read() {
                        MessageReceiver::new().process_message(
                            guid_prefix,
                            &*rtps_participant.builtin_reader_group.lock(),
                            source_locator,
                            &message,
                        );
                    }

                    let writer_group = rtps_participant.builtin_writer_group.lock();
                    for writer in writer_group.writer_list() {
                        let mut writer = writer.lock();
                        send_data(
                            protocol_version,
                            vendor_id,
                            guid_prefix,
                            &mut *writer,
                            &mut transport,
                        );
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        let domain_participant = DomainParticipantImpl::new(rtps_participant_impl, is_enabled);

        Some(domain_participant)
    }
}
