#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

extern crate alloc;
#[cfg(feature = "dcps")]
mod dds;

#[cfg(feature = "dcps")]
pub use dds::*;

/// Contains the async version of the DDS API.
#[cfg(feature = "dcps")]
pub mod dds_async;

/// Contains the DCPS logic which provides the behavior to the DDS API
#[doc(hidden)]
#[cfg(feature = "dcps")]
pub mod dcps;

#[cfg(feature = "dcps")]
pub use dcps::{builtin_topics, infrastructure};

#[doc(hidden)]
#[cfg(feature = "rtps")]
pub mod rtps;

#[cfg(feature = "rtps_messages")]
#[doc(hidden)]
pub mod rtps_messages;

#[cfg(feature = "rtps_udp_transport")]
#[doc(hidden)]
pub mod rtps_udp_transport;

#[cfg(feature = "transport")]
#[doc(hidden)]
/// Contains the Dust DDS transport interface definition.
pub mod transport;

#[cfg(feature = "dcps")]
/// Contains the Dust DDS runtime abstractions.
pub mod runtime;

#[cfg(feature = "std")]
#[doc(hidden)]
pub mod std_runtime;

/// Contains the XTypes serializer and deserializer
#[cfg(feature = "xtypes")]
#[doc(hidden)]
pub mod xtypes;

// To enable using our own derive macros to allow the name dust_dds:: to be used
extern crate self as dust_dds;
