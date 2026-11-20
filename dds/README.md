# Dust DDS

[![CircleCI](https://dl.circleci.com/status-badge/img/gh/s2e-systems/dust-dds/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/s2e-systems/dust-dds/tree/main)
[![Crates.io](https://img.shields.io/crates/v/dust_dds.svg)](https://crates.io/crates/dust_dds)
[![Docs.rs](https://docs.rs/dust_dds/badge.svg)](https://docs.rs/dust_dds)
[![License](https://img.shields.io/crates/l/dust_dds.svg)](https://github.com/s2e-systems/dust-dds/blob/main/LICENSE)
[![WebRTC](https://img.shields.io/badge/WebRTC%20%26%20WASM-Browser%20Native-blue)](#browser-connectivity-via-webassembly--webrtc)
[![OMG DDS Security](https://img.shields.io/badge/OMG%20DDS%20Security%20v1.1-WIP-orange)](#omg-dds-security-v11-plugin-work-in-progress)
[![Services](https://img.shields.io/badge/Engineering%20Services-S2E%20Systems-green)](https://www.s2e-systems.com)

[Dust DDS](https://dust-dds.s2e-systems.com) is a native, memory-safe Rust implementation of the [OMG Data Distribution Service (DDS)](https://www.omg.org/omg-dds-portal/) middleware and [Real-time Publisher-Subscriber (RTPS)](https://www.omg.org/spec/DDSI-RTPS/About-DDSI-RTPS/) wire protocol standard developed by [S2E Software Systems](https://www.s2e-systems.com).

Dust DDS provides peer-to-peer, data-centric connectivity for Physical AI, robotics, defense, aerospace, and mission-critical embedded systems. It uses only stable Rust with `#![forbid(unsafe_code)]` in the core engine, guaranteeing deterministic, zero-overhead execution without garbage collection pauses or C/C++ memory corruption vulnerabilities.

---

## Key Architectural Features

* **Brokerless Peer-to-Peer Discovery**: Nodes dynamically discover each other over the network via standard RTPS discovery. There is no central broker to configure, bottleneck, or introduce a single point of failure.
* **Fine-Grained Quality of Service (QoS)**: Contract-based data delivery policies built natively into the wire protocol (Reliability, Durability, Deadline, Liveliness, History, and Ownership).
* **Deterministic Memory Safety**: Pure, stable Rust eliminates memory safety CVEs, buffer overflows, and data races in multi-threaded stacks without runtime garbage collector pauses.
* **Modular Pluggable Transports**: Configurable transport architecture supporting standard UDP/IP (v4 and v6), zero-copy shared memory (`iceoryx2`), and WebRTC DataChannels.
* **Multi-Platform Support**: Runs on desktop Linux, macOS, and Windows, safety-critical RTOSes (QNX, VxWorks), resource-constrained microcontrollers (`no_std`), and web browsers via WebAssembly.

---

## Comparison with Alternative DDS Implementations

| Feature | Dust DDS | Eclipse Cyclone DDS | eProsima Fast DDS | RTI Connext |
| :--- | :---: | :---: | :---: | :---: |
| **Primary Language** | Safe Rust | C | C++ | C / C++ |
| **Memory Safety** | Compile-time guaranteed | Manual / C liabilities | Manual / C++ liabilities | Manual / C++ liabilities |
| **Security Dependencies** | Zero OpenSSL bloat (Pure Rust) | OpenSSL | OpenSSL | OpenSSL / WolfSSL |
| **Browser / WebRTC** | Native WASM & WebRTC DataChannels | External bridges only | External bridges only | External gateways |
| **Pluggable Transports** | Modular trait (UDP, Shared Memory, WebRTC) | Static / Monolithic | Transport API | Pluggable (Monolithic core) |
| **Embedded & Microcontrollers** | Native `no_std` / Embassy support | CycloneDDS Micro | Micro XRCE-DDS | Connext Micro / Cert |
| **License** | Apache 2.0 | EPL 2.0 | Apache 2.0 | Commercial / Proprietary |

---

## Browser Connectivity via WebAssembly & WebRTC

Standard DDS implementations rely on UDP multicast and raw UDP sockets, which cannot be opened within web browsers. Integrating DDS with browser-based teleoperation consoles or diagnostic dashboards traditionally required intermediate WebSocket proxies (such as `rosbridge`) or dedicated VPN tunnels, which introduce serialization overhead, head-of-line blocking, and complex deployment requirements over cellular networks.

Dust DDS addresses this by compiling directly to WebAssembly (`wasm32-unknown-unknown`) and communicating over WebRTC DataChannels:

```text
┌────────────────────────────────────┐              WebSocket Signaling (Discovery Only)
│   Web Browser Client               │< ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ >┌────────────────────────────────────┐
│   (Chrome / Firefox / Safari)      │                                                          │   Native Node / Robot              │
│                                    │            WebRTC DataChannel ("dust_rtps")              │   (dust_dds daemon / application)  │
│   • Pure Rust via dust_dds_wasm    │=========================================================>│                                    │
│   • Zero client installation       │              Sub-100ms, zero-copy RTPS data              │   • Native UDP / Shared Memory     │
│   • Runs in Web Worker or main UI  │<=========================================================│   • STUN/TURN NAT Traversal        │
└────────────────────────────────────┘                                                          └────────────────────────────────────┘
```

* **Zero-Install Web Clients**: The WebAssembly package (`dust_dds_wasm`) runs directly inside standard browsers, enabling low-latency telemetry visualization and control interfaces delivered via URL.
* **Direct RTPS DataChannels**: RTPS messages travel directly over peer-to-peer WebRTC DataChannels, retaining binary serialization and fine-grained DDS QoS contracts.
* **NAT & Firewall Traversal**: Built-in STUN/TURN integration enables reliable connectivity across symmetric NATs and cellular (4G/5G) links without public IP addresses or client VPN software.

A complete runnable demonstration of bidirectional communication between a native Rust application and a web browser client is available in [bindings/wasm/demo](https://github.com/s2e-systems/dust-dds/tree/main/bindings/wasm/demo).

---

## Example

### Publisher

```rust
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{listener::NO_LISTENER, qos::QosKind, status::NO_STATUS, type_support::DdsType},
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

### Subscriber

```rust
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{listener::NO_LISTENER, qos::QosKind, status::NO_STATUS, type_support::DdsType},
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

---

## Defining DDS Types

Dust DDS allows topic types to be defined natively in Rust using the `DdsType` derive macro:

```rust
use dust_dds::infrastructure::type_support::DdsType;

#[derive(DdsType)]
struct SensorData {
    #[dust_dds(key)]
    sensor_id: u32,
    timestamp: u64,
    reading: f64,
}
```

DDS types can also be generated from OMG IDL v4 specifications using the [`dust_dds_gen`](https://crates.io/crates/dust_dds_gen) crate.

For full technical documentation on:
* Container attributes (`name`, `extensibility`, `nested`, `base_type`)
* Field attributes (`key`, `id`, `optional`, `default_value`, `non_serialized`, `external`, `hashid`, `try_construct`)
* Single inheritance (`base_type` and `parent` field mapping)
* Enumerated types (`bit_bound`) and XTypes Unions (`switch`, `case`, `default`)
* IDL type generation

Please refer to the [DDS Type Definition and IDL Guide](https://github.com/s2e-systems/dust-dds/blob/main/dds/docs/type_support.md).

---

## Synchronous and Asynchronous API

Dust DDS provides both synchronous and asynchronous APIs to support diverse application architectures:

* **Synchronous API**: Follows the standard DDS specification using DDS listeners and waitsets for event-driven architectures. Recommended for standard embedded and multi-threaded systems.
* **Asynchronous API**: Built for async runtimes such as [Tokio](https://tokio.rs). When integrating with Tokio, the asynchronous API must be used to avoid panics resulting from blocking calls inside the runtime.

---

## Multi-Language and Ecosystem Bindings

* **WebAssembly (`dust_dds_wasm`)**: WebAssembly bindings running in browsers with WebRTC DataChannels for teleoperation and telemetry.
* **Python Bindings (`dust-dds-python`)**: Connects AI training and inference pipelines (PyTorch, JAX) and simulation platforms directly to DDS topics without IPC conversion overhead.
* **C / C++ Bindings**: Stable C-compatible API allowing incremental integration into existing C and C++ codebases.
* **ROS 2 RMW (`rmw_dust_dds`)**: Drop-in ROS 2 RMW implementation supporting active ROS 2 distributions (Humble, Jazzy).
* **Microcontrollers (`no_std`)**: Support for bare-metal targets and microcontrollers with a pre-built runtime using the Embassy async framework.

---

## Commercial Extensions & Engineering Services

[S2E Software Systems](https://www.s2e-systems.com) offers commercial software modules, platform porting, and engineering services for production deployments of Dust DDS:

### OMG DDS Security v1.1 Plugin (Work in Progress)
A pure-Rust implementation of the OMG DDS Security v1.1 specification, delivering cryptographic protection without OpenSSL dependencies:
* **Authentication**: PKI mutual authentication with X.509 certificates.
* **Access Control**: Fine-grained, signed topic governance and permissions.
* **Wire Cryptography**: AES-GCM (128/256-bit) payload encryption and message authentication (MAC).
* *Status*: Under active development. Contact S2E for roadmap access and pilot evaluation.

### WebRTC Teleoperation Fleet Gateway
Production edge daemon and signaling infrastructure connecting local DDS networks to remote browser consoles across enterprise NATs and cellular links.

### Custom Transport Implementation
Design and qualification of custom transport drivers for specialized hardware buses and determinism requirements:
* Time-Sensitive Networking (TSN / IEEE 802.1Qbv)
* CAN / CAN-FD and industrial fieldbuses
* PCIe interconnects and FPGA shared-memory buffers
* Tactical radio, SATCOM, and custom RF links

### Platform & RTOS Porting
Porting, hardening, and qualification of Dust DDS for customer hardware targets and certified operating environments:
* Safety-critical RTOSes (QNX Neutrino, Wind River VxWorks)
* Small-footprint targets (Zephyr RTOS, FreeRTOS, bare-metal `no_std`)
* Board bring-up with automated Hardware-in-the-Loop (HIL) qualification

### Technical Training & Support SLAs
* **Workshops**: Hands-on engineering workshops covering DDS QoS tuning, distributed systems architecture, and Rust integration into legacy C/C++ stacks.
* **Commercial Support**: Commercial SLAs, dedicated issue resolution, and long-term support (LTS) backports available upon request.

For inquiries, contact [info@s2e-systems.com](mailto:info@s2e-systems.com) or visit [s2e-systems.com](https://www.s2e-systems.com).

---

## Shapes Demo

DDS interoperability is typically validated using a shapes demo. The Dust DDS Shapes Demo is included in this repository and can be launched with:

```bash
cargo run --package dust_dds_shapes_demo
```

![Dust DDS Shapes demo screenshot](https://raw.githubusercontent.com/s2e-systems/dust-dds/main/dds/docs/shapes_demo_screenshot.png)

---

## License

This project is licensed under the Apache License Version 2.0.
