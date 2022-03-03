use std::iter::FromIterator;

use rust_dds_api::dcps_psm::{InstanceStateKind, ViewStateKind};
use rust_rtps_pim::{
    messages::{
        submessage_elements::Parameter,
        types::{ParameterId, Time},
    },
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        history_cache::{
            RtpsHistoryCacheAttributes, RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations,
        },
        types::{ChangeKind, Guid, InstanceHandle, SequenceNumber},
    },
};
pub struct WriterCacheChange {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub sequence_number: SequenceNumber,
    pub instance_handle: InstanceHandle,
    pub data: Vec<u8>,
    pub _source_timestamp: Option<Time>,
    pub _view_state_kind: ViewStateKind,
    pub _instance_state_kind: InstanceStateKind,
    pub inline_qos: RtpsParameterList,
}

#[derive(Debug, PartialEq)]
pub struct RtpsParameter {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: Vec<u8>,
}

pub struct RtpsParameterList(pub Vec<RtpsParameter>);
impl<'a> IntoIterator for &'a RtpsParameterList {
    type Item = Parameter<'a>;
    type IntoIter = std::vec::IntoIter<Parameter<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        let v: Vec<Parameter> = self
            .0
            .iter()
            .map(|p| Parameter {
                parameter_id: p.parameter_id,
                length: p.length,
                value: p.value.as_ref(),
            })
            .collect();
        v.into_iter()
    }
}
impl<'a> FromIterator<&'a Parameter<'a>> for RtpsParameterList {
    fn from_iter<T: IntoIterator<Item = &'a Parameter<'a>>>(iter: T) -> Self {
        Self(iter.into_iter().map(|p| RtpsParameter {
            parameter_id: p.parameter_id,
            length: p.length,
            value: p.value.to_vec(),
        }).collect())
    }
}

impl RtpsParameter {
    pub fn new(parameter_id: ParameterId, value: &[u8]) -> Self {
        let length = ((value.len() + 3) & !0b11) as i16; //ceil to multiple of 4;
        Self {
            parameter_id,
            length,
            value: value.to_vec(),
        }
    }
}

impl<'a> RtpsCacheChangeConstructor<'a> for WriterCacheChange {
    type DataType = [u8];
    type ParameterListType = [Parameter<'a>];

    fn new(
        kind: &ChangeKind,
        writer_guid: &Guid,
        instance_handle: &InstanceHandle,
        sequence_number: &SequenceNumber,
        data_value: &Self::DataType,
        _inline_qos: &Self::ParameterListType,
    ) -> Self {
        Self {
            kind: *kind,
            writer_guid: *writer_guid,
            sequence_number: *sequence_number,
            instance_handle: *instance_handle,
            data: data_value.to_vec(),
            _source_timestamp: None,
            _view_state_kind: ViewStateKind::New,
            _instance_state_kind: InstanceStateKind::Alive,
            inline_qos: RtpsParameterList(vec![]),
        }
    }
}

impl RtpsCacheChangeAttributes<'_> for WriterCacheChange {
    type DataType = [u8];
    type ParameterListType = RtpsParameterList;

    fn kind(&self) -> &ChangeKind {
        &self.kind
    }

    fn writer_guid(&self) -> &Guid {
        &self.writer_guid
    }

    fn instance_handle(&self) -> &InstanceHandle {
        &self.instance_handle
    }

    fn sequence_number(&self) -> &SequenceNumber {
        &self.sequence_number
    }

    fn data_value(&self) -> &Self::DataType {
        self.data.as_ref()
    }

    fn inline_qos(&self) -> &Self::ParameterListType {
        &self.inline_qos
    }
}

pub struct WriterHistoryCache {
    changes: Vec<WriterCacheChange>,
    source_timestamp: Option<Time>,
}

impl WriterHistoryCache {
    /// Set the Rtps history cache impl's info.
    pub fn set_source_timestamp(&mut self, info: Option<Time>) {
        self.source_timestamp = info;
    }
}

impl RtpsHistoryCacheConstructor for WriterHistoryCache {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
            source_timestamp: None,
        }
    }
}

impl RtpsHistoryCacheAttributes for WriterHistoryCache {
    type CacheChangeType = WriterCacheChange;

    fn changes(&self) -> &[Self::CacheChangeType] {
        &self.changes
    }
}

impl RtpsHistoryCacheOperations for WriterHistoryCache {
    type CacheChangeType = WriterCacheChange;

    fn add_change(&mut self, change: Self::CacheChangeType) {
        self.changes.push(change)
    }

    fn remove_change(&mut self, seq_num: &SequenceNumber) {
        self.changes.retain(|cc| &cc.sequence_number != seq_num)
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.changes
            .iter()
            .map(|cc| cc.sequence_number)
            .min()
            .clone()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.changes
            .iter()
            .map(|cc| cc.sequence_number)
            .max()
            .clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rust_rtps_pim::structure::types::GUID_UNKNOWN;

    #[test]
    fn remove_change() {
        let mut hc = WriterHistoryCache::new();
        let change = WriterCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &vec![],
            &vec![],
        );
        hc.add_change(change);
        hc.remove_change(&1);
        assert!(hc.changes().is_empty());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc = WriterHistoryCache::new();
        let change1 = WriterCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &vec![],
            &vec![],
        );
        let change2 = WriterCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &2,
            &vec![],
            &vec![],
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_min(), Some(1));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc = WriterHistoryCache::new();
        let change1 = WriterCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &1,
            &vec![],
            &vec![],
        );
        let change2 = WriterCacheChange::new(
            &rust_rtps_pim::structure::types::ChangeKind::Alive,
            &GUID_UNKNOWN,
            &0,
            &2,
            &vec![],
            &vec![],
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(2));
    }
}
