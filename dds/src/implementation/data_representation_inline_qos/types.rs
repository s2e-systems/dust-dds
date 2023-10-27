use crate::{cdr::serialize::CdrSerialize, topic_definition::cdr_type::CdrDeserialize};

#[derive(Clone, Copy, CdrSerialize, CdrDeserialize, PartialEq, Eq)]
pub struct KeyHash(pub [u8; 16]);

#[derive(Clone, Copy, CdrSerialize, CdrDeserialize, PartialEq, Eq)]
pub struct StatusInfo([u8; 4]);
pub const STATUS_INFO_DISPOSED: StatusInfo = StatusInfo([0, 0, 0, 0b00000001]);
pub const STATUS_INFO_UNREGISTERED: StatusInfo = StatusInfo([0, 0, 0, 0b0000010]);
pub const STATUS_INFO_DISPOSED_UNREGISTERED: StatusInfo = StatusInfo([0, 0, 0, 0b00000011]);
pub const _STATUS_INFO_FILTERED: StatusInfo = StatusInfo([0, 0, 0, 0b0000100]);
