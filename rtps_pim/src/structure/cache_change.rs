use crate::messages::submessage_elements::Parameter;

use super::types::{ChangeKind, InstanceHandle, SequenceNumber, Guid};

pub struct RtpsCacheChange<'a> {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub instance_handle: InstanceHandle,
    pub sequence_number: SequenceNumber,
    pub data_value: &'a [u8],
    pub inline_qos: &'a [Parameter<'a>],
}