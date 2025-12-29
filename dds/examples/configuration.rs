use dust_dds::{
    configuration::DustDdsConfigurationBuilder,
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{qos::QosKind, status::NO_STATUS, type_support::DdsType},
    listener::NO_LISTENER,
};
use std::time::Duration;

#[derive(DdsType, Debug)]
struct HelloWorldType {
    #[dust_dds(key)]
    id: u8,
    msg: String,
}

fn main() {
    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    // Configure DustDDS with custom settings:
    // - domain_tag: Tag for domain isolation (default: "")
    // - participant_announcement_interval: How often to announce presence via SPDP (default: 5s)
    // - participant_lease_duration: How long before a silent participant is considered dead (default: 100s)
    let configuration = DustDdsConfigurationBuilder::new()
        .domain_tag("abc".to_string())
        .participant_announcement_interval(Duration::from_secs(5))
        .participant_lease_duration(Duration::from_secs(100))
        .build()
        .unwrap();

    participant_factory
        .set_configuration(configuration)
        .unwrap();

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
