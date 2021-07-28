use crate::messages::submessage_elements::Parameter;

use super::types::{ChangeKind, InstanceHandle, SequenceNumber, GUID};

pub struct RtpsCacheChange<'a> {
    kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: InstanceHandle,
    sequence_number: SequenceNumber,
    data_value: &'a [u8],
    inline_qos: &'a [Parameter<'a>],
}

impl<'a> RtpsCacheChange<'a> {
    pub fn new(
        kind: ChangeKind,
        writer_guid: GUID,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        data_value: &'a [u8],
        inline_qos: &'a [Parameter<'a>],
    ) -> Self {
        Self {
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data_value,
            inline_qos,
        }
    }

    pub fn kind(&self) -> &ChangeKind {
        &self.kind
    }

    /// Get a reference to the rtps cache change's writer guid.
    pub fn writer_guid(&self) -> &GUID {
        &self.writer_guid
    }

    /// Get a reference to the rtps cache change's instance handle.
    pub fn instance_handle(&self) -> &InstanceHandle {
        &self.instance_handle
    }

    /// Get a reference to the rtps cache change's sequence number.
    pub fn sequence_number(&self) -> &SequenceNumber {
        &self.sequence_number
    }

    /// Get a reference to the rtps cache change's data value.
    pub fn data_value(&self) -> &&'a [u8] {
        &self.data_value
    }

    /// Get a reference to the rtps cache change's inline qos.
    pub fn inline_qos(&self) -> &&'a [Parameter<'a>] {
        &self.inline_qos
    }
}
