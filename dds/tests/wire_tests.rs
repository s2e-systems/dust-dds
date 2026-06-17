mod utils;
use std::sync::OnceLock;

use dust_dds::{
    dds_async::domain_participant_factory::DomainParticipantFactoryAsync,
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{listener::NO_LISTENER, qos::QosKind, status::NO_STATUS},
    rtps_messages::{
        overall_structure::RtpsMessageWrite,
        submessage_elements::{Data, ParameterList},
        submessages::data::DataSubmessage,
    },
    transport::{
        interface::{
            RtpsTransportParticipant, TransportDataReceiver, TransportParticipantFactory,
            WriteMessage,
        },
        types::{BUILT_IN_WRITER_WITH_KEY, ENTITYID_UNKNOWN, EntityId, Locator},
    },
};

use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[test]
fn detect_stale_participant() {
    struct MockWriter;
    impl WriteMessage for MockWriter {
        fn write_message(&self, _buf: &[u8], _locators: &[Locator]) {}
    }

    struct MockTransport(std::sync::mpsc::SyncSender<TransportDataReceiver>);
    impl TransportParticipantFactory for MockTransport {
        fn create_participant(
            &self,
            _domain_id: i32,
            data_receiver: TransportDataReceiver,
        ) -> RtpsTransportParticipant {
            self.0.send(data_receiver).unwrap();
            RtpsTransportParticipant {
                message_writer: Box::new(MockWriter),
                default_unicast_locator_list: Vec::new(),
                metatraffic_unicast_locator_list: Vec::new(),
                metatraffic_multicast_locator_list: Vec::new(),
                default_multicast_locator_list: Vec::new(),
                fragment_size: 1000,
            }
        }
    }
    static PARTICIPANT_FACTORY_ASYNC: OnceLock<DomainParticipantFactoryAsync<MockTransport>> =
        OnceLock::new();

    let (data_receiver_send, data_receiver_recv) = std::sync::mpsc::sync_channel(1);
    let transport = MockTransport(data_receiver_send);
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let domain_participant_factory =
        DomainParticipantFactory::new(PARTICIPANT_FACTORY_ASYNC.get_or_init(|| {
            let executor = dust_dds::std_runtime::executor::Executor::new();
            let timer_driver = dust_dds::std_runtime::timer::TimerDriver::new();
            let runtime = dust_dds::std_runtime::StdRuntime::new(executor, timer_driver);
            let app_id = [1, 2, 3, 4];
            let host_id = [5, 6, 7, 8];
            let configuration = Default::default();

            DomainParticipantFactoryAsync::new(runtime, app_id, host_id, transport, configuration)
        }));

    let participant = domain_participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let guid_prefix = <[u8; 16]>::from(participant.get_instance_handle())[0..12]
        .try_into()
        .unwrap();

    let data_receiver = data_receiver_recv.recv().unwrap();

    let reader_id = ENTITYID_UNKNOWN;
    const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId =
        EntityId::new([0x00, 0x01, 0x00], BUILT_IN_WRITER_WITH_KEY);
    let writer_id = ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER;
    let inline_qos_flag = false;
    let data_flag = true;
    let key_flag = false;
    let non_standard_payload_flag = false;
    let writer_sn = 1;
    let inline_qos = ParameterList::empty();
    let serialized_payload = Data::new(
        vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x50, 0x00, 16, 0x00, // PID_PARTICIPANT_GUID, Length
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            8, 8, 8, 8, // GuidPrefix
            0, 0, 1, 0xc1, // EntityId
            0x15, 0x00, 4, 0x00, // PID_PROTOCOL_VERSION, Length
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion
            0x16, 0x00, 4, 0x00, // PID_VENDORID
            73, 74, 0x00, 0x00, // VendorId
            88, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_SET
            0x02, 0x00, 0x00, 0x00, //
            0x02, 0x00, 8, 0x00, // PID_PARTICIPANT_LEASE_DURATION
            1, 0x00, 0x00, 0x00, // Duration: seconds
            0, 0x00, 0x00, 0x00, // Duration: fraction
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL
        ]
        .into(),
    );
    let spdp_data_submessage = DataSubmessage::new(
        inline_qos_flag,
        data_flag,
        key_flag,
        non_standard_payload_flag,
        reader_id,
        writer_id,
        writer_sn,
        inline_qos,
        serialized_payload,
    );
    let spdp_rtps_message =
        RtpsMessageWrite::from_submessages(&[&spdp_data_submessage], guid_prefix);

    dust_dds::std_runtime::executor::block_on(
        data_receiver.receive_message(spdp_rtps_message.buffer().to_vec()),
    );

    assert_eq!(participant.get_discovered_participants().unwrap().len(), 1);

    // Wait longer than lease duration communicated in the discovery message
    std::thread::sleep(std::time::Duration::from_secs(2));

    assert_eq!(participant.get_discovered_participants().unwrap().len(), 0);
}
