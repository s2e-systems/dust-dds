use crate::types::GUID;

pub trait RTPSEntity {
    type GUID: GUID;
}
