#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![doc = include_str!("../README.md")]
mod dds;
/// Contains the traits and classes needed to represent types in the formats defined in the RTPS standard
pub mod serialized_payload;
pub use dds::*;
/// Contains the async version of the DDS API.
pub mod dds_async;

#[doc(hidden)]
pub mod rtps;

#[doc(hidden)]
pub mod data_representation_builtin_endpoints;

mod implementation;

// To enable using our own derive macros
extern crate self as dust_dds;
