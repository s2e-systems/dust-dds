use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        configuration::DustDdsConfigurationBuilder, listener::NO_LISTENER, qos::QosKind,
        status::NO_STATUS, type_support::DdsType,
    },
    rtps_udp_transport::RtpsUdpTransport,
    security::plugins::types::DdsSecurityPlugins,
};

#[derive(DdsType, Debug)]
struct HelloWorldType {
    #[dust_dds(key)]
    id: u8,
    msg: String,
}

fn main() {
    let domain_id = 0;
    let configuration = DustDdsConfigurationBuilder::new()
        .domain_tag("abc".to_string())
        .build()
        .unwrap();

    let participant_factory = DomainParticipantFactory::get_custom_instance(
        configuration,
        RtpsUdpTransport::default(),
        DdsSecurityPlugins::disabled(),
    );

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
