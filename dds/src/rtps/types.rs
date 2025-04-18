use crate::{
    infrastructure::qos_policy::{
        DurabilityQosPolicy, DurabilityQosPolicyKind, ReliabilityQosPolicy,
        ReliabilityQosPolicyKind,
    },
    transport::types::{DurabilityKind, ProtocolVersion, ReliabilityKind, VendorId},
};

// This files shall only contain the types as listed in the DDSI-RTPS Version 2.5
// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes

impl From<&ReliabilityQosPolicy> for ReliabilityKind {
    fn from(value: &ReliabilityQosPolicy) -> Self {
        match value.kind {
            ReliabilityQosPolicyKind::BestEffort => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::Reliable => ReliabilityKind::Reliable,
        }
    }
}

impl From<&DurabilityQosPolicy> for DurabilityKind {
    fn from(value: &DurabilityQosPolicy) -> Self {
        match value.kind {
            DurabilityQosPolicyKind::Volatile => DurabilityKind::Volatile,
            DurabilityQosPolicyKind::TransientLocal => DurabilityKind::TransientLocal,
            DurabilityQosPolicyKind::Transient => DurabilityKind::Transient,
            DurabilityQosPolicyKind::Persistent => DurabilityKind::Persistent,
        }
    }
}

// Defined elsewhere in DDS

pub const PROTOCOLVERSION: ProtocolVersion = PROTOCOLVERSION_2_4;
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
pub const PROTOCOLVERSION_2_4: ProtocolVersion = ProtocolVersion::new(2, 4);

#[allow(dead_code)]
pub const VENDOR_ID_UNKNOWN: VendorId = [0, 0];
pub const VENDOR_ID_S2E: VendorId = [0x01, 0x14];
