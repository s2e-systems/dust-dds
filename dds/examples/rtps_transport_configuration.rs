use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{qos::QosKind, status::NO_STATUS, type_support::DdsType},
    listener::NO_LISTENER,
    rtps_udp_transport::udp_transport::RtpsUdpTransportParticipantFactoryBuilder,
};

#[derive(DdsType, Debug)]
struct HelloWorldType {
    #[dust_dds(key)]
    id: u8,
    msg: String,
}

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();
    let transport = Box::new(
        RtpsUdpTransportParticipantFactoryBuilder::new()
            .fragment_size(500)
            .build()
            .unwrap(),
    );

    participant_factory.set_transport(transport).unwrap();

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

    writer.write(&hello_world, None).unwrap();
}
