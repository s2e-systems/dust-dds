#![allow(dead_code)]

// mod cache;
// mod entity;
// mod guid;
mod inline_qos;
mod messages;
// mod participant;
// mod participant_proxy;
// mod proxy;
// mod reader;
// mod transport;
pub mod types;
// mod writer;
// mod stateless_reader;
// mod stateless_writer;
mod serdes;

// pub use stateless_reader::StatelessReader;
// pub use stateless_writer::StatelessWriter;

// pub enum LocatorKind {
//     LocatorInvalid,
//     Invalid,
//     Reserved,
//     Udpv4,
//     Udpv6,
//     LocatorAddressInvalid,
//     LocatorPortInvalid,
// }

// trait Locator {}

// pub struct Udpv4Locator {
//     pub kind: LocatorKind,
//     pub address: [u8; 4],
//     pub port: u16,
// }

// impl Udpv4Locator {
//     pub fn new_udpv4(address: &[u8; 4], port: &u16) -> Udpv4Locator {
//         Udpv4Locator {
//             kind: LocatorKind::Udpv4,
//             address: *address,
//             port: *port,
//         }
//     }
// }

// impl Locator for Udpv4Locator {}
