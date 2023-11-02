use dust_dds::{
    configuration::DustDdsConfigurationBuilder,
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{listeners::NoOpListener, qos::QosKind, status::NO_STATUS},
    topic_definition::type_support::DdsType,
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
    let configuration = DustDdsConfigurationBuilder::new()
        .domain_tag("abc".to_string())
        .unwrap()
        .fragment_size(1000)
        .unwrap()
        .interface_name(None)
        .unwrap()
        .build();
    participant_factory
        .set_configuration(configuration)
        .unwrap();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic(
            "HelloWorld",
            "HelloWorldType",
            QosKind::Default,
            NoOpListener::new(),
            NO_STATUS,
        )
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let writer = publisher
        .create_datawriter(&topic, QosKind::Default, NoOpListener::new(), NO_STATUS)
        .unwrap();

    let hello_world = HelloWorldType {
        id: 8,
        msg: "Hello world!".to_string(),
    };

    writer.write(&hello_world, None).unwrap();
}
