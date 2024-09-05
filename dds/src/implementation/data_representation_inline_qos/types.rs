use dust_dds_xtypes::serialize::XTypesSerialize;

use crate::serialized_payload::cdr::deserialize::CdrDeserialize;

#[derive(Clone, Copy, CdrDeserialize, PartialEq, Eq, XTypesSerialize)]
pub struct KeyHash(pub [u8; 16]);

#[derive(Clone, Copy, CdrDeserialize, PartialEq, Eq, XTypesSerialize)]
pub struct StatusInfo([u8; 4]);
pub const STATUS_INFO_DISPOSED: StatusInfo = StatusInfo([0, 0, 0, 0b00000001]);
pub const STATUS_INFO_UNREGISTERED: StatusInfo = StatusInfo([0, 0, 0, 0b0000010]);
pub const STATUS_INFO_DISPOSED_UNREGISTERED: StatusInfo = StatusInfo([0, 0, 0, 0b00000011]);
pub const _STATUS_INFO_FILTERED: StatusInfo = StatusInfo([0, 0, 0, 0b0000100]);
