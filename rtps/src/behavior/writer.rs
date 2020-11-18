use crate::structure::RtpsEndpoint;

use rust_dds_interface::types::{SequenceNumber, ChangeKind, InstanceHandle, ParameterList};
use rust_dds_interface::cache_change::CacheChange;

pub struct RtpsWriter {
    pub endpoint: RtpsEndpoint,
    pub push_mode: bool,
    pub last_change_sequence_number: SequenceNumber,
    pub data_max_sized_serialized: Option<i32>,
}

impl RtpsWriter {
    pub fn new(
        endpoint: RtpsEndpoint,
        push_mode: bool,
        data_max_sized_serialized: Option<i32>,
    ) -> Self {
        Self {
            endpoint,
            push_mode,
            last_change_sequence_number: 0,
            data_max_sized_serialized,
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
        handle: InstanceHandle,
    ) -> CacheChange {
        self.last_change_sequence_number += 1;
        CacheChange::new(
            kind,
            self.endpoint.entity.guid.into(),
            handle,
            self.last_change_sequence_number,
            data,
            inline_qos,
        )
    }
}
