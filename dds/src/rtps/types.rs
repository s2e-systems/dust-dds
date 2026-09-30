use alloc::{string::String, vec::Vec};
use dust_dds_derive::TypeSupport;

use crate::transport::types::{ProtocolVersion, VendorId};

pub const PROTOCOLVERSION: ProtocolVersion = PROTOCOLVERSION_2_5;
#[allow(dead_code)]
pub const PROTOCOLVERSION_1_0: ProtocolVersion = ProtocolVersion::new(1, 0);
#[allow(dead_code)]
pub const PROTOCOLVERSION_1_1: ProtocolVersion = ProtocolVersion::new(1, 1);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_0: ProtocolVersion = ProtocolVersion::new(2, 0);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_1: ProtocolVersion = ProtocolVersion::new(2, 1);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_2: ProtocolVersion = ProtocolVersion::new(2, 2);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_3: ProtocolVersion = ProtocolVersion::new(2, 3);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_4: ProtocolVersion = ProtocolVersion::new(2, 4);
pub const PROTOCOLVERSION_2_5: ProtocolVersion = ProtocolVersion::new(2, 5);

#[allow(dead_code)]
pub const VENDOR_ID_UNKNOWN: VendorId = [0, 0];
pub const VENDOR_ID_S2E: VendorId = [0x01, 0x14];

/// Property type as defined in Section 9.3.2 of the DDS-RTPS specification and extended by DDS Security.
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct Property {
    /// Name of the property.
    pub name: String,
    /// Value associated with that name.
    pub value: String,
    /// Indicates whether the property is intended for local use only or should be propagated by DDS discovery.
    #[dust_dds(non_serialized)]
    pub propagate: bool,
}

/// Sequence of [`Property`].
pub type PropertySeq = Vec<Property>;
