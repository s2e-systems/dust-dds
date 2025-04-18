#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

extern crate alloc;
#[cfg(feature = "std")]
mod dds;

#[cfg(feature = "std")]
pub use dds::*;
/// Contains the async version of the DDS API.
#[cfg(feature = "std")]
pub mod dds_async;

#[doc(hidden)]
#[cfg(feature = "std")]
pub mod rtps;

#[cfg(feature = "rtps_messages")]
#[doc(hidden)]
pub mod rtps_messages;

#[cfg(feature = "std")]
#[doc(hidden)]
pub mod rtps_udp_transport;

#[cfg(feature = "std")]
mod implementation;

#[cfg(feature = "transport")]
#[doc(hidden)]
/// Contains the Dust DDS transport interface definition.
pub mod transport;

#[cfg(feature = "std")]
mod runtime;

/// Contains the XTypes serializer and deserializer
#[cfg(feature = "xtypes")]
#[doc(hidden)]
pub mod xtypes;

// To enable using our own derive macros to allow the name dust_dds:: to be used
extern crate self as dust_dds;
