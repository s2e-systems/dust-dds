pub(crate) mod deserializer;
pub(crate) mod read_write;
pub(crate) mod serializer;

#[doc(hidden)]
pub mod bytes;

#[doc(hidden)]
pub mod data_storage;

/// Classes related to the dynamic representation of types and data.
pub mod dynamic_type;

/// Classes related to the XTypes error and return codes.
pub mod error;

/// Type Object representation as defined in the DDS-XTypes 1.3 standard.
///
/// A TypeObject describes a type structure completely or minimally, allowing DDS participants
/// to exchange type information dynamically during discovery.
pub mod type_object;

/// Traits related to the representation of XTypes required to be transmitted using DDS
pub mod type_support;
