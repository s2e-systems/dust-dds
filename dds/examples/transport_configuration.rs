use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        listener::NO_LISTENER, qos::QosKind, status::NO_STATUS, type_support::DdsType,
    },
    rtps_udp_transport::udp_transport::RtpsUdpTransport,
};

#[derive(DdsType, Debug)]
struct HelloWorldType {
    #[dust_dds(key)]
    id: u8,
    msg: String,
}

fn main() {
    let mut transport = RtpsUdpTransport::default();
    transport
        .set_interface_name(Some(String::from("Wi-Fi")))
        .set_fragment_size(500)
        .unwrap()
        .set_udp_receive_buffer_size(Some(10000));

    let participant_factory =
        DomainParticipantFactory::get_custom_instance(transport, Default::default());

    let domain_id = 0;
    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<HelloWorldType>(
            "HelloWorld",
            "HelloWorldType",
            QosKind::Default,
            NO_LISTENER,
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer = publisher
        .create_datawriter(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let hello_world = HelloWorldType {
        id: 8,
        msg: "Hello world!".to_string(),
    };

    writer.write(hello_world, None).unwrap();
}
