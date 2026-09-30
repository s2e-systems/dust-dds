use alloc::{string::String, vec::Vec};
use dust_dds_derive::TypeSupport;

/// BinaryProperty type as defined in Section 7.3.3 of the DDS Security specification.
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct BinaryProperty {
    /// Name of the binary property.
    pub name: String,
    /// Value associated with that name.
    pub value: Vec<u8>,
    /// Indicates whether the binary property is intended for local use only or should be propagated by DDS discovery.
    #[dust_dds(non_serialized)]
    pub propagate: bool,
}

/// Sequence of [`BinaryProperty`].
pub type BinaryPropertySeq = Vec<BinaryProperty>;
