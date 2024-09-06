#![forbid(unsafe_code)]
#![forbid(missing_docs)]

#![cfg_attr(not(feature = "std"), no_std)]

#![doc = include_str!("../README.md")]
#[cfg(feature = "std")]
mod dds;
/// Contains the traits and classes needed to represent types in the formats defined in the RTPS standard
#[cfg(feature = "std")]
pub mod serialized_payload;
#[cfg(feature = "std")]
pub use dds::*;
/// Contains the async version of the DDS API.
#[cfg(feature = "std")]
pub mod dds_async;

#[doc(hidden)]
#[cfg(feature = "std")]
pub mod rtps;

#[doc(hidden)]
#[cfg(feature = "std")]
pub mod data_representation_builtin_endpoints;

#[cfg(feature = "std")]
mod implementation;

/// Contains the XTypes serializer and deserializer
#[doc(hidden)]
pub mod xtypes;

// To enable using our own derive macros to allow the name dust_dds:: to be used
extern crate self as dust_dds;
