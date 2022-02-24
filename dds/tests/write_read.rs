use std::net::{UdpSocket, SocketAddr};

use rust_dds::{domain_participant_factory::DomainParticipantFactory, infrastructure::{qos::{DomainParticipantQos, DataReaderQos}, entity::Entity, qos_policy::{ReliabilityQosPolicyKind}}, domain::domain_participant::DomainParticipant, publication::{publisher::Publisher, data_writer::DataWriter}, subscription::{subscriber::Subscriber, data_reader::DataReader}, types::Time, udp_transport::UdpTransport, communication::Communication};
use rust_dds_rtps_implementation::{dds_type::{DdsSerialize, DdsType, DdsDeserialize}, rtps_impl::{rtps_reader_proxy_impl::RtpsReaderProxyAttributesImpl, rtps_writer_proxy_impl::RtpsWriterProxyImpl}};
use rust_rtps_pim::{structure::{types::{PROTOCOLVERSION, VENDOR_ID_S2E, GuidPrefix, Locator, LOCATOR_KIND_UDPv4}, entity::RtpsEntityAttributes, endpoint::RtpsEndpointAttributes}, behavior::{reader::{writer_proxy::RtpsWriterProxyConstructor
    , reader::RtpsReaderAttributes, stateful_reader::RtpsStatefulReaderOperations}, writer::{writer::RtpsWriterAttributes, stateful_writer::RtpsStatefulWriterOperations}}};
use rust_rtps_pim::behavior::writer::reader_proxy::RtpsReaderProxyConstructor;

struct MyType {}

impl DdsType for MyType {
    fn type_name() -> &'static str {
        "MyType"
    }

    fn has_key() -> bool {
        false
    }
}

impl DdsSerialize for MyType {
    fn serialize<W: std::io::Write, E: rust_dds_rtps_implementation::dds_type::Endianness>(
        &self,
        _writer: W,
    ) -> rust_dds::DDSResult<()> {
        Ok(())
    }
}

impl<'de> DdsDeserialize<'de> for MyType {
    fn deserialize(_buf: &mut &'de [u8]) -> rust_dds::DDSResult<Self> {
        Ok(MyType {})
    }
}

fn user_unicast_communication() -> Communication<UdpTransport> {
    let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 7410))).unwrap();

    Communication {
        version: PROTOCOLVERSION,
        vendor_id: VENDOR_ID_S2E,
        guid_prefix: GuidPrefix([3; 12]),
        transport: UdpTransport::new(socket),
    }
}

#[test]
fn user_defined_write_read() {
    let participant_factory = DomainParticipantFactory::get_instance();

    let mut qos = DomainParticipantQos::default();
    qos.entity_factory.autoenable_created_entities = false;

    let participant1 = participant_factory
        .create_participant(0, Some(qos.clone()), None, 0)
        .unwrap();
    participant1.enable().unwrap();

    let participant2 = participant_factory
        .create_participant(0, Some(qos.clone()), None, 0)
        .unwrap();
    participant2.enable().unwrap();

    let topic = participant1
        .create_topic::<MyType>("MyTopic", None, None, 0)
        .unwrap();

    let publisher = participant1.create_publisher(None, None, 0).unwrap();
    let mut writer = publisher.create_datawriter(&topic, None, None, 0).unwrap();

    let mut reader_qos = DataReaderQos::default();
    reader_qos.reliability.kind = ReliabilityQosPolicyKind::ReliableReliabilityQos;
    let subscriber = participant2.create_subscriber(None, None, 0).unwrap();
    let mut reader = subscriber.create_datareader(&topic, Some(reader_qos), None, 0).unwrap();

    {
        let rtps_writer_shared = writer.as_ref().upgrade().unwrap();
        let rtps_writer = &mut rtps_writer_shared.write_lock().rtps_writer;
        let stateful_writer = rtps_writer.try_as_stateful_writer().unwrap();
        let writer_proxy = RtpsWriterProxyImpl::new(
            stateful_writer.guid().clone(),
            &[Locator::new(
                LOCATOR_KIND_UDPv4,
                7410,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
            )],
            stateful_writer.multicast_locator_list(),
            stateful_writer.data_max_size_serialized().clone(),
            stateful_writer.guid().entity_id,
        );

        let rtps_reader_shared = reader.as_ref().upgrade().unwrap();
        let rtps_reader = &mut rtps_reader_shared.write_lock().rtps_reader;
        let stateful_reader = rtps_reader.try_as_stateful_reader().unwrap();
        let reader_proxy = RtpsReaderProxyAttributesImpl::new(
            stateful_reader.guid().clone(),
            stateful_reader.guid().entity_id,
            &[Locator::new(
                LOCATOR_KIND_UDPv4,
                7410,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1],
            )],
            stateful_reader.multicast_locator_list(),
            *stateful_reader.expects_inline_qos(),
            true,
        );

        stateful_writer.matched_reader_add(reader_proxy);
        stateful_reader.matched_writer_add(writer_proxy);
    }

    writer
        .write_w_timestamp(&MyType {}, None, Time { sec: 0, nanosec: 0 })
        .unwrap();

    let mut user_communication = user_unicast_communication();
    user_communication.send(&[publisher.as_ref().upgrade().unwrap()]);
    user_communication.receive(&[subscriber.as_ref().upgrade().unwrap()]);

    let samples = reader.read(1, &[], &[], &[]).unwrap();
    assert!(samples.len() == 1);
}