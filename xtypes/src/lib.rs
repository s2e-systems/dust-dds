#![no_std]

/// Contains the trait for the CDR Deserialize.
pub mod deserialize;
/// Contains the trait for the CDR Deserializer.
pub mod deserializer;
/// Contains the trait for the CDR Serialize.
pub mod serialize;
/// Contains the trait for the CDR Serializer.
pub mod serializer;

pub mod error;

pub mod xcdr_deserializer;
pub mod xcdr_serializer;
