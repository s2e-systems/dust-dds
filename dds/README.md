# Dust DDS

Dust DDS is a native [Rust](https://www.rust-lang.org/) implementation of the OMG [Data Distribution Services (DDS)](https://www.omg.org/omg-dds-portal/) and [Real-time Publisher-Subscriber (RTPS)](https://www.omg.org/spec/DDSI-RTPS/About-DDSI-RTPS/) developed by [S2E Software Systems](https://www.s2e-systems.com).

The goal of this crate is to provide a high-quality Rust implementation of the minimum DDS profile. For high-quality it is meant that the implementation is done using stable Rust and without unsafe code and with large unit test code coverage.

## A brief introduction to DDS

DDS is a middleware protocol and API standard designed for data-centric connectivity. At its core, DDS aims to facilitate the seamless sharing of pertinent data precisely where and when it's needed, even across publishers and subscribers operating asynchronously in time. With DDS, applications can exchange information through the reading and writing of data-objects identified by user-defined names (Topics) and keys. One of its defining features is the robust control it offers over Quality-of-Service (QoS) parameters, encompassing reliability, bandwidth, delivery deadlines, and resource allocations.

The [DDS standard](https://www.omg.org/spec/DDS/1.4/PDF) "defines both the Application Interfaces (APIs) and the Communication Semantics (behavior and quality of service) that enable the efficient delivery of information from information producers to matching consumer". Complementing this standard is the [DDSI-RTPS specification](https://www.omg.org/spec/DDSI-RTPS/2.5/PDF), which defines an interoperability wire protocol for DDS. Its primary aim is to ensure that applications based on different vendors' implementations of DDS can interoperate. The implementation of Dust DDS primarily centers around the DDS and DDSI-RTPS standards.

## Example

A basic example on how to use Dust DDS. The publisher side can be implemented as:

```rust
    use dust_dds::{
        domain::domain_participant_factory::DomainParticipantFactory,
        infrastructure::{qos::QosKind, status::NO_STATUS},
        topic_definition::type_support::DdsType,
    };

    #[derive(DdsType)]
    struct HelloWorldType {
        #[dust_dds(key)]
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
            .create_topic::<HelloWorldType>("HelloWorld", "HelloWorldType", QosKind::Default, None, NO_STATUS)
            .unwrap();

        let publisher = participant
            .create_publisher(QosKind::Default, None, NO_STATUS)
            .unwrap();

        let writer = publisher
            .create_datawriter::<HelloWorldType>(&topic, QosKind::Default, None, NO_STATUS)
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
        topic_definition::type_support::DdsType,
    };

    #[derive(Debug, DdsType)]
    struct HelloWorldType {
        #[dust_dds(key)]
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
            .create_topic::<HelloWorldType>("HelloWorld", "HelloWorldType", QosKind::Default, None, NO_STATUS)
            .unwrap();

        let subscriber = participant
            .create_subscriber(QosKind::Default, None, NO_STATUS)
            .unwrap();

        let reader = subscriber
            .create_datareader::<HelloWorldType>(&topic, QosKind::Default, None, NO_STATUS)
            .unwrap();

        let samples = reader
            .read(1, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE);

        if let Ok(hello_world_samples) = samples {
            println!("Received: {:?}", hello_world_samples[0].data().unwrap());
        }
    }
```

## IDL type generation

If using only Rust, you can make use of the procedural macros to enable a type to be transmitted
using DustDDS. The key fields can also be defined as part of the macro.

```rust
use dust_dds::topic_definition::type_support::DdsType;

#[derive(DdsType)]
struct HelloWorldType {
    #[dust_dds(key)]
    id: u8,
    msg: String,
}
```

If using different programming languages or vendors, the DDS type can be generated from an OMG IDL file using the [dust_dds_gen crate](https://crates.io/crates/dust_dds_gen).

## Sync and Async library API

Dust DDS provides both a "sync" and an "async" API to allow integrating DDS in the largest number of applications with maximum performance. In general, the first option should be to use the sync API and make use of the DDS specified functionality such as listeners for event based programs.

When implementing applications that already make use of async, then the async API must be used. In particular, when using a Tokio runtime, using the Sync API will result in a panic due to blocking calls. You can see find an example in the examples folder.

## DDS REST API

If you want to interact with your DDS data using a REST API you can use our [Nebula DDS WebLink](https://www.s2e-systems.com/products/nebula-dds-weblink/) software. Nebula DDS WebLink provides a server implementing the Object Management Group (OMG) Web-Enabled DDS v1.0 standard.

## Shapes demo

DDS interoperability is typically tested using a shapes demo. The Dust DDS Shapes Demo is available on our repository and can be started by running `cargo run --package dust_dds_shapes_demo` from the root folder.

![Dust DDS Shapes demo screenshot](https://raw.githubusercontent.com/s2e-systems/dust-dds/master/dds/docs/shapes_demo_screenshot.png)

## Release schedule

Dust DDS doesn't follow a fixed release schedule but we will make releases as new features are implemented.

## License

This project is licensed under the Apache License Version 2.0.
