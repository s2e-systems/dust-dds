use rtps_pim::{
    messages::{submessage_elements::Parameter, types::ParameterId},
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        history_cache::{
            RtpsHistoryCacheAttributes, RtpsHistoryCacheConstructor, RtpsHistoryCacheOperations,
        },
        types::{ChangeKind, Guid, InstanceHandle, SequenceNumber},
    },
};

#[derive(Debug, PartialEq)]
pub struct RtpsParameter {
    pub parameter_id: ParameterId,
    pub value: Vec<u8>,
}

pub struct RtpsParameterList(pub Vec<RtpsParameter>);

impl<'a> From<&'a Vec<Parameter<'_>>> for RtpsParameterList {
    fn from(p: &'a Vec<Parameter<'_>>) -> Self {
        Self(
            p.into_iter()
                .map(|p| RtpsParameter {
                    parameter_id: p.parameter_id,
                    value: p.value.to_vec(),
                })
                .collect(),
        )
    }
}
impl<'a> From<&'a RtpsParameterList> for Vec<Parameter<'a>> {
    fn from(v: &'a RtpsParameterList) -> Self {
        v.0.iter()
            .map(|p| Parameter {
                parameter_id: p.parameter_id,
                length: p.value.len() as i16,
                value: p.value.as_ref(),
            })
            .collect()
    }
}

pub struct RtpsCacheChangeImpl {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub sequence_number: SequenceNumber,
    pub instance_handle: InstanceHandle,
    pub data: RtpsData,
    pub inline_qos: RtpsParameterList,
}

pub struct RtpsData(pub Vec<u8>);
impl From<&&[u8]> for RtpsData {
    fn from(v: &&[u8]) -> Self {
        Self(v.to_vec())
    }
}
impl<'a> From<&'a RtpsData> for &'a [u8] {
    fn from(v: &'a RtpsData) -> Self {
        v.0.as_ref()
    }
}
impl AsRef<[u8]> for RtpsData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl RtpsCacheChangeConstructor for RtpsCacheChangeImpl {
    type DataType = RtpsData;
    type ParameterListType = RtpsParameterList;

    fn new(
        kind: ChangeKind,
        writer_guid: Guid,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        data_value: Self::DataType,
        inline_qos: Self::ParameterListType,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            sequence_number,
            instance_handle,
            data: data_value,
            inline_qos: inline_qos.into(),
        }
    }
}

impl RtpsCacheChangeAttributes<'_> for RtpsCacheChangeImpl {
    type DataType = [u8];
    type ParameterListType = RtpsParameterList;

    fn kind(&self) -> ChangeKind {
        self.kind
    }

    fn writer_guid(&self) -> Guid {
        self.writer_guid
    }

    fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    fn data_value(&self) -> &Self::DataType {
        self.data.as_ref()
    }

    fn inline_qos(&self) -> &Self::ParameterListType {
        &self.inline_qos
    }
}

pub struct RtpsHistoryCacheImpl {
    changes: Vec<RtpsCacheChangeImpl>,
}

impl RtpsHistoryCacheConstructor for RtpsHistoryCacheImpl {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }
}

impl RtpsHistoryCacheAttributes for RtpsHistoryCacheImpl {
    type CacheChangeType = RtpsCacheChangeImpl;

    fn changes(&self) -> &[Self::CacheChangeType] {
        &self.changes
    }
}

impl RtpsHistoryCacheOperations for RtpsHistoryCacheImpl {
    type CacheChangeType = RtpsCacheChangeImpl;

    fn add_change(&mut self, change: Self::CacheChangeType) {
        self.changes.push(change)
    }

    fn remove_change<F>(&mut self, mut f: F)
    where
        F: FnMut(&Self::CacheChangeType) -> bool,
    {
        self.changes.retain(|cc| !f(cc))
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
    use rtps_pim::structure::types::GUID_UNKNOWN;

    #[test]
    fn remove_change() {
        let mut hc = RtpsHistoryCacheImpl::new();
        let change = RtpsCacheChangeImpl::new(
            rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            RtpsData(vec![]),
            RtpsParameterList(vec![]),
        );
        hc.add_change(change);
        hc.remove_change(|cc| cc.sequence_number() == 1);
        assert!(hc.changes().is_empty());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc = RtpsHistoryCacheImpl::new();
        let change1 = RtpsCacheChangeImpl::new(
            rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            RtpsData(vec![]),
            RtpsParameterList(vec![]),
        );
        let change2 = RtpsCacheChangeImpl::new(
            rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            2,
            RtpsData(vec![]),
            RtpsParameterList(vec![]),
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_min(), Some(1));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc = RtpsHistoryCacheImpl::new();
        let change1 = RtpsCacheChangeImpl::new(
            rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            RtpsData(vec![]),
            RtpsParameterList(vec![]),
        );
        let change2 = RtpsCacheChangeImpl::new(
            rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            2,
            RtpsData(vec![]),
            RtpsParameterList(vec![]),
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(2));
    }
}
