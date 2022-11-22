use crate::{
    infrastructure::{instance::InstanceHandle, time::Time},
    subscription::sample_info::{SampleStateKind, ViewStateKind},
};

use super::{
    history_cache::RtpsParameter,
    types::{ChangeKind, Guid, SequenceNumber},
};

#[derive(Debug, Clone)]

pub struct RtpsReaderCacheChange {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    instance_handle: InstanceHandle,
    data: Vec<u8>,
    _inline_qos: Vec<RtpsParameter>,
    source_timestamp: Option<Time>,
    sample_state: SampleStateKind,
    view_state: ViewStateKind,
}

impl PartialEq for RtpsReaderCacheChange {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.writer_guid == other.writer_guid
            && self.sequence_number == other.sequence_number
            && self.instance_handle == other.instance_handle
    }
}

impl RtpsReaderCacheChange {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kind: ChangeKind,
        writer_guid: Guid,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        data_value: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        source_timestamp: Option<Time>,
        view_state: ViewStateKind,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            sequence_number,
            instance_handle,
            data: data_value,
            _inline_qos: inline_qos,
            source_timestamp,
            sample_state: SampleStateKind::NotRead,
            view_state,
        }
    }

    pub fn kind(&self) -> ChangeKind {
        self.kind
    }

    pub fn writer_guid(&self) -> Guid {
        self.writer_guid
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    pub fn data_value(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn source_timestamp(&self) -> &Option<Time> {
        &self.source_timestamp
    }

    pub fn sample_state(&self) -> SampleStateKind {
        self.sample_state
    }

    pub fn view_state(&self) -> ViewStateKind {
        self.view_state
    }

    pub fn mark_read(&mut self) {
        self.sample_state = SampleStateKind::Read;
    }
}
