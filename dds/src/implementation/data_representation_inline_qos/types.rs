use dust_dds_derive::XTypesDeserialize;

use crate::xtypes::serialize::XTypesSerialize;

#[derive(Clone, Copy, PartialEq, Eq, XTypesSerialize, XTypesDeserialize, Debug)]
pub struct KeyHash(pub [u8; 16]);

#[derive(Clone, Copy, PartialEq, Eq, XTypesSerialize, XTypesDeserialize, Debug)]
pub struct StatusInfo(pub [u8; 4]);
pub const STATUS_INFO_DISPOSED: StatusInfo = StatusInfo([0, 0, 0, 0b00000001]);
pub const STATUS_INFO_UNREGISTERED: StatusInfo = StatusInfo([0, 0, 0, 0b0000010]);
pub const STATUS_INFO_DISPOSED_UNREGISTERED: StatusInfo = StatusInfo([0, 0, 0, 0b00000011]);
pub const STATUS_INFO_FILTERED: StatusInfo = StatusInfo([0, 0, 0, 0b0000100]);
