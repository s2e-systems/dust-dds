# Dust DDS

Dust DDS is a native [Rust](https://www.rust-lang.org/) implementation of the [Data Distribution Services (DDS)](https://www.omg.org/omg-dds-portal/) using the [Real-time Publisher-Subscriber (RTPS)](https://www.omg.org/spec/DDSI-RTPS/About-DDSI-RTPS/) wire protocol developed by [S2E Software Systems](https://www.s2e-systems.com).

This crate provides a Rust implementation of the minimum DDS profile. It uses only stable Rust and has no `unsafe` code while providing a large code coverage validated by our CI systems to ensure its quality.

## A brief introduction to DDS

DDS is a machine-to-machine communication middleware and API standard designed for data-centric connectivity. At its core, DDS aims to facilitate the seamless sharing of pertinent data precisely where and when it's needed, even across publishers and subscribers operating asynchronously in time. With DDS, applications can exchange information through the reading and writing of data-objects identified by user-defined names (Topics) and keys. One of its defining features is the robust control it offers over Quality-of-Service (QoS) parameters, encompassing reliability, bandwidth, delivery deadlines, and resource allocations.

The [DDS standard](https://www.omg.org/spec/DDS/1.4/PDF) "defines both the Application Interfaces (APIs) and the Communication Semantics (behavior and quality of service) that enable the efficient delivery of information from information producers to matching consumer". Complementing this standard is the [DDSI-RTPS specification](https://www.omg.org/spec/DDSI-RTPS/2.5/PDF), which defines an interoperability wire protocol for DDS. Its primary aim is to ensure that applications based on different vendors' implementations of DDS can interoperate. The implementation of Dust DDS primarily centers around the DDS and DDSI-RTPS standards.

## Who should use DDS?

Choosing the right middleware depends on many factors like *network architecture*, *latency requirements*, *scalability* and *workload*. There are though some questions you can ask to determine if DDS is the right solution:
1. Do I have a dynamic network topology? A dynamic network topology means that devices are able to join and leave the communication domain at anytime. Imagine an air traffic control system, a rail network monitoring system or a robot with variable sensor and actuator setup. This makes any fixed configuration very challenging and is well handled by the dynamic discovery system of DDS.
2. Do I need to send a large volume of messages and/or operate on the millisecond time range? Imagine publishing data from a radar system, monitoring a vehicle position or sending mixed robot video and sensor data. Default DDS implementation use UDP and provide an extensive QoS which enable it to handle high-throughput and low-latency messaging.
3. Does my system need to be robust to failures on the different nodes? Imagine a defense system which should be resilient or a robot that can continue to operate when certain sensors or actuators are removed. DDS discovery does not rely on any centralized server and communication is peer-to-peer. This reduces single points-of-failure and bottlenecks.
4. Do I need well defined communication types that can evolve over time? Imagine an automotive or robotic system where you want to have well defined data types transmitted by the sensors. DDS is a data-centric middleware and its type system allows type definitions to be extended and evolve over time.

If you answered yes to one or more of these questions, DDS is likely a strong candidate for your application. There are also certain common applications for which DDS is typically not the best fit. These include:

1. Heavy database-centric workloads. If your system is primarily used to store and retrieve data from persistent data storage you are probably better off with a database-driven approach.
2. Non-real-time web applications. If your main requirement is scalable centralized web-based communication you are probably better of with other message broker solutions
3. Simple request-response workloads. If your application follows a standard request-response model then you are probably better off with REST or RPC-based communication even if DDS is able to handle request reply mechanisms.

## Example

A basic example on how to use Dust DDS. The publisher side can be implemented as:

```rust
    use dust_dds::{
        domain::domain_participant_factory::DomainParticipantFactory,
        listener::NO_LISTENER,
        infrastructure::{qos::QosKind, status::NO_STATUS, type_support::DdsType},
    };

    #[derive(DdsType)]
    struct HelloWorldType {
        #[dust_dds(key)]
        id: u8,
        msg: String,
    }

    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<HelloWorldType>("HelloWorld", "HelloWorldType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let writer = publisher
        .create_datawriter::<HelloWorldType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let hello_world = HelloWorldType {
        id: 8,
        msg: "Hello world!".to_string(),
    };
    writer.write(hello_world, None).unwrap();
```

The subscriber side can be implemented as:

```rust
    use dust_dds::{
        domain::domain_participant_factory::DomainParticipantFactory,
        listener::NO_LISTENER,
        infrastructure::{qos::QosKind, status::NO_STATUS, type_support::DdsType},
    };

    #[derive(Debug, DdsType)]
    struct HelloWorldType {
        #[dust_dds(key)]
        id: u8,
        msg: String,
    }

    let domain_id = 0;
    let participant_factory = DomainParticipantFactory::get_instance();

    let participant = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let topic = participant
        .create_topic::<HelloWorldType>("HelloWorld", "HelloWorldType", QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let subscriber = participant
        .create_subscriber(QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let reader = subscriber
        .create_datareader::<HelloWorldType>(&topic, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    if let Ok(hello_world_sample) = reader.read_next_sample() {
        println!("Received: {:?}", hello_world_sample.data.unwrap());
    }
```

## IDL type generation

If using only Rust, you can make use of the procedural macros to enable a type to be transmitted
using Dust DDS. The key fields can also be defined as part of the macro.

```rust
use dust_dds::infrastructure::type_support::DdsType;

#[derive(DdsType)]
struct HelloWorldType {
    #[dust_dds(key)]
    id: u8,
    msg: String,
}
```

If using different programming languages or vendors, the DDS type can be generated from an IDL file using the [dust_dds_gen crate](https://crates.io/crates/dust_dds_gen). The `HelloWorldType` can be generated from the following idl.

```idl
struct HelloWorldType {
  @key
  uint8 id;
  string msg;
};
```

## Sync and Async library API

Dust DDS provides both a "sync" and an "async" API to allow integrating DDS in the largest number of applications with maximum performance. In general, the first option should be to use the sync API and make use of the DDS specified functionality such as listeners for event based programs.

When implementing applications that already make use of async, then the async API must be used. In particular, when using a Tokio runtime, using the Sync API will result in a panic due to blocking calls. You can see find an example in the examples folder.

## Dynamic type discovery

Dust DDS supports runtime type discovery via the DDS-XTypes TypeLookup service. This allows applications to subscribe to topics without compile-time type knowledge, which is useful for generic monitoring tools, data recorders, or dynamic language bindings.

The `type_lookup` feature is enabled by default. To disable it for size-constrained builds, use `default-features = false` in your Cargo.toml.

```rust
use dust_dds::{
    dds_async::domain_participant_factory::DomainParticipantFactoryAsync,
    infrastructure::{qos::QosKind, status::NO_STATUS},
};

let participant_factory = DomainParticipantFactoryAsync::get_instance();
let participant = participant_factory
    .create_participant(0, QosKind::Default, None::<()>, NO_STATUS)
    .await
    .unwrap();

let subscriber = participant
    .create_subscriber(QosKind::Default, None::<()>, NO_STATUS)
    .await
    .unwrap();

// Discover the type from a remote publisher
let dynamic_type = participant.discover_type("HelloWorld").await.unwrap();

// Create a reader using the discovered type
let reader = subscriber
    .create_dynamic_datareader("HelloWorld", dynamic_type, QosKind::Default)
    .await
    .unwrap();

// Read data and access fields by name
for sample in reader.take(10, &ANY_SAMPLE_STATE, &ANY_VIEW_STATE, &ANY_INSTANCE_STATE).await.unwrap() {
    if let Some(data) = sample.data {
        let id = data.get_uint8_value_by_name("id").unwrap();
        let msg = data.get_string_value_by_name("msg").unwrap();
        println!("Received: id={}, msg={}", id, msg);
    }
}
```

## Dust DDS extensions

### DDS over the Internet
Standard DDS implementations are limited to local networks because they rely on UDP multicast for participant discovery, which is not permitted across the internet. If you want to connect your devices over the internet, our [Global DDS](https://www.s2e-systems.com/products/global-dds/) offers a plug-and-play solution without code modifications or additional software needed.

### DDS REST API

If you want to interact with your DDS data using a REST API you can use our [Nebula DDS WebLink](https://www.s2e-systems.com/products/nebula-dds-weblink/) software. Nebula DDS WebLink provides a server implementing the Object Management Group (OMG) Web-Enabled DDS v1.0 standard.

## Shapes demo

DDS interoperability is typically tested using a shapes demo. The Dust DDS Shapes Demo is available on our repository and can be started by running `cargo run --package dust_dds_shapes_demo` from the root folder.

![Dust DDS Shapes demo screenshot](https://raw.githubusercontent.com/s2e-systems/dust-dds/master/dds/docs/shapes_demo_screenshot.png)

## Release schedule

Dust DDS doesn't follow a fixed release schedule but we will make releases as new features are implemented.

## License

This project is licensed under the Apache License Version 2.0.
