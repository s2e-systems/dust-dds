#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct KeyHash(pub [u8; 16]);

pub type StatusInfo = u32;
pub const STATUS_INFO_DISPOSED_FLAG: StatusInfo = 0x0001 << 0;
pub const STATUS_INFO_UNREGISTERED_FLAG: StatusInfo = 0x0001 << 1;
pub const STATUS_INFO_FILTERED_FLAG: StatusInfo = 0x0001 << 2;
