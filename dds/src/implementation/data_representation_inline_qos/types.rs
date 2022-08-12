#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct KeyHash(pub [u8; 16]);

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct StatusInfo([u8; 4]);
pub const STATUS_INFO_DISPOSED_FLAG: StatusInfo = StatusInfo([0, 0, 0, 0x0001 << 0]);
pub const STATUS_INFO_UNREGISTERED_FLAG: StatusInfo = StatusInfo([0, 0, 0, 0x0001 << 1]);
pub const _STATUS_INFO_FILTERED_FLAG: StatusInfo = StatusInfo([0, 0, 0, 0x0001 << 2]);
