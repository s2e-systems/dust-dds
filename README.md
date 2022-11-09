# Dust DDS

This repository contains the [S2E Software Systems](https://www.s2e-systems.com) implementation of the OMG [Data Distribution Services (DDS)](https://www.omg.org/omg-dds-portal/) and [Real-time Publisher-Subscriber (RTPS)](https://www.omg.org/spec/DDSI-RTPS/About-DDSI-RTPS/) protocols using the [Rust programming language](https://www.rust-lang.org/).

The aim is to provide a high-quality Rust implementation of the minimum DDS profile. For high-quality it is meant that the implementation is done using stable Rust and without unsafe code and with large unit test code coverage.

***Note: This crate is a work-in-progress and so far only the most basic functionality is expected to be working***
## Example

A basic example on how to use Dust DDS. Make sure you import the crate into your

```toml
[dependencies]
dust_dds = "0.1.0"
```

Then the publisher side can be implemented as:

```rust
    use dust_dds::{
        domain::domain_participant_factory::DomainParticipantFactory,
        infrastructure::{qos::QosKind, status::NO_STATUS},
        topic_definition::type_support::{DdsSerde, DdsType},
    };

    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, DdsType, DdsSerde)]
    struct HelloWorldType {
        #[key]
        id: u8,
        msg: String,
    }

    fn main() {
        let domain_id = 0;
        let participant_factory = DomainParticipantFactory::get_instance();

        let participant = participant_factory
            .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
            .unwrap();

        let topic = participant
            .create_topic::<HelloWorldType>("HelloWorld", QosKind::Default, None, NO_STATUS)
            .unwrap();

        let publisher = participant
            .create_publisher(QosKind::Default, None, NO_STATUS)
            .unwrap();

        let writer = publisher
            .create_datawriter(&topic, QosKind::Default, None, NO_STATUS)
            .unwrap();

        let hello_world = HelloWorldType {
            id: 8,
            msg: "Hello world!".to_string(),
        };
        writer.write(&hello_world, None).unwrap();
    }
```

The subscriber side can be implemented as:

```rust
    use dust_dds::{
        domain::domain_participant_factory::DomainParticipantFactory,
        infrastructure::{qos::QosKind, status::NO_STATUS},
        subscription::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        topic_definition::type_support::{DdsSerde, DdsType},
    };

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, DdsType, DdsSerde)]
    struct HelloWorldType {
        #[key]
        id: u8,
        msg: String,
    }

    fn main() {
        let domain_id = 0;
        let participant_factory = DomainParticipantFactory::get_instance();

        let participant = participant_factory
            .create_participant(domain_id, QosKind::Default, None, NO_STATUS)
            .unwrap();

        let topic = participant
            .create_topic::<HelloWorldType>("HelloWorld", QosKind::Default, None, NO_STATUS)
            .unwrap();

        let subscriber = participant
            .create_subscriber(QosKind::Default, None, NO_STATUS)
            .unwrap();

        let reader = subscriber
            .create_datareader(&topic, QosKind::Default, None, NO_STATUS)
            .unwrap();

        let samples = reader
            .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .unwrap();

        let hello_world = samples[0].data.as_ref().unwrap();
        println!("Received: {:?}", hello_world);
    }
```

## Release schedule

Dust DDS doesn't follow a fixed release schedule but we will make releases as new features are implemented. Until we are out of the experimental phase we will not provide separate bugfix versions.

## License

This project is licensed under the Apache License Version 2.0.