use alloc::sync::Arc;

use super::error::RtpsError;
use crate::{
    dcps::data_representation_builtin_endpoints::parameter_id_values::{
        PID_COHERENT_SET, PID_GROUP_COHERENT_SET, PID_GROUP_SEQ_NUM, PID_WRITER_GROUP_INFO,
    },
    rtps::types::{ChangeKind, SequenceNumber, Time, WriterGroupInfo},
    rtps_messages::{
        self,
        overall_structure::TryReadFromBytes,
        submessage_elements::ParameterListWrite,
        submessages::{
            data::{DataSubmessage, DataSubmessageWrite},
            data_frag::DataFragSubmessageWrite,
        },
        types::{GroupDigest, ParameterId},
    },
    transport::types::{EntityId, Guid, GuidPrefix},
};

pub const PID_KEY_HASH: ParameterId = 0x0070;
pub const PID_STATUS_INFO: ParameterId = 0x0071;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheChangeRead {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub sequence_number: i64,
    pub source_timestamp: Option<Time>,
    pub instance_handle: Option<[u8; 16]>,
    pub data_value: Arc<[u8]>,
    pub coherent_set: Option<SequenceNumber>,
    pub group_coherent_set: Option<SequenceNumber>,
    pub group_seq_num: Option<SequenceNumber>,
    pub writer_group_info: Option<WriterGroupInfo>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StatusInfo(pub [u8; 4]);
pub const STATUS_INFO_DISPOSED: StatusInfo = StatusInfo([0, 0, 0, 0b00000001]);
pub const STATUS_INFO_UNREGISTERED: StatusInfo = StatusInfo([0, 0, 0, 0b0000010]);
pub const STATUS_INFO_DISPOSED_UNREGISTERED: StatusInfo = StatusInfo([0, 0, 0, 0b00000011]);
pub const STATUS_INFO_FILTERED: StatusInfo = StatusInfo([0, 0, 0, 0b0000100]);

use crate::rtps_messages::submessage_elements::ParameterList;

pub struct CacheChangeParameters {
    pub change_kind: ChangeKind,
    pub instance_handle: Option<[u8; 16]>,
    pub coherent_set: Option<SequenceNumber>,
    pub group_coherent_set: Option<SequenceNumber>,
    pub group_seq_num: Option<SequenceNumber>,
    pub writer_group_info: Option<WriterGroupInfo>,
}

impl TryFrom<&ParameterList<'_>> for CacheChangeParameters {
    type Error = RtpsError;

    fn try_from(value: &ParameterList<'_>) -> Result<Self, Self::Error> {
        let change_kind = match value
            .parameters()
            .find(|x| x.parameter_id() == PID_STATUS_INFO)
        {
            Some(p) => {
                if p.length() == 4 {
                    let status_info =
                        StatusInfo([p.value()[0], p.value()[1], p.value()[2], p.value()[3]]);
                    match status_info {
                        STATUS_INFO_DISPOSED => Ok(ChangeKind::NotAliveDisposed),
                        STATUS_INFO_UNREGISTERED => Ok(ChangeKind::NotAliveUnregistered),
                        STATUS_INFO_DISPOSED_UNREGISTERED => {
                            Ok(ChangeKind::NotAliveDisposedUnregistered)
                        }
                        STATUS_INFO_FILTERED => Ok(ChangeKind::AliveFiltered),
                        _ => Err(RtpsError::InvalidData),
                    }
                } else {
                    Err(RtpsError::InvalidData)
                }
            }
            None => Ok(ChangeKind::Alive),
        }?;

        let instance_handle = match value
            .parameters()
            .find(|x| x.parameter_id() == PID_KEY_HASH)
        {
            Some(p) => <[u8; 16]>::try_from(p.value()).ok(),

            None => None,
        };

        let coherent_set = match value
            .parameters()
            .find(|x| x.parameter_id() == PID_COHERENT_SET)
        {
            Some(x) => Some(SequenceNumber::try_read_from_bytes(
                &mut x.value(),
                &value.endianness(),
            )?),
            None => None,
        };

        let group_coherent_set = match value
            .parameters()
            .find(|x| x.parameter_id() == PID_GROUP_COHERENT_SET)
        {
            Some(x) => Some(SequenceNumber::try_read_from_bytes(
                &mut x.value(),
                &value.endianness(),
            )?),
            None => None,
        };

        let group_seq_num = match value
            .parameters()
            .find(|x| x.parameter_id() == PID_GROUP_SEQ_NUM)
        {
            Some(x) => Some(SequenceNumber::try_read_from_bytes(
                &mut x.value(),
                &value.endianness(),
            )?),
            None => None,
        };

        let writer_group_info = match value
            .parameters()
            .find(|x| x.parameter_id() == PID_WRITER_GROUP_INFO)
        {
            Some(x) => Some(WriterGroupInfo {
                writer_set: GroupDigest::try_read_from_bytes(&mut x.value(), &value.endianness())?,
            }),
            None => None,
        };

        Ok(Self {
            change_kind,
            instance_handle,
            coherent_set,
            group_coherent_set,
            group_seq_num,
            writer_group_info,
        })
    }
}

impl CacheChangeRead {
    pub fn try_from_data_submessage(
        data_submessage: &DataSubmessage<'_>,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<rtps_messages::types::Time>,
    ) -> Result<Self, RtpsError> {
        let CacheChangeParameters {
            change_kind,
            instance_handle,
            coherent_set,
            group_coherent_set,
            group_seq_num,
            writer_group_info,
        } = data_submessage.inline_qos().try_into()?;

        Ok(Self {
            kind: change_kind,
            writer_guid: Guid::new(source_guid_prefix, data_submessage.writer_id()),
            source_timestamp: source_timestamp.map(Into::into),
            instance_handle,
            sequence_number: data_submessage.writer_sn(),
            data_value: data_submessage.serialized_payload().into(),
            coherent_set,
            group_coherent_set,
            group_seq_num,
            writer_group_info,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheChangeWrite {
    pub writer_guid: Guid,
    pub sequence_number: i64,
    pub source_timestamp: Time,
    pub data_value: Arc<[u8]>,
    pub inline_qos: ParameterListWrite,
}

impl CacheChangeWrite {
    pub fn as_data_submessage<'a>(
        &'a self,
        reader_id: EntityId,
        writer_id: EntityId,
    ) -> DataSubmessageWrite<'a> {
        let (data_flag, key_flag) = if self.data_value.is_empty() {
            (false, false)
        } else {
            match self.inline_qos.kind {
                ChangeKind::Alive | ChangeKind::AliveFiltered => (true, false),
                ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered => (false, true),
            }
        };

        let inline_qos_flag = true;

        DataSubmessageWrite::new(
            inline_qos_flag,
            data_flag,
            key_flag,
            false,
            reader_id,
            writer_id,
            self.sequence_number,
            self.inline_qos,
            self.data_value.as_ref(),
        )
    }

    pub fn as_data_frag_submessage<'a>(
        &'a self,
        reader_id: EntityId,
        writer_id: EntityId,
        data_max_size_serialized: usize,
        fragment_number: usize,
    ) -> DataFragSubmessageWrite<'a> {
        let inline_qos_flag = fragment_number == 0;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let writer_sn = self.sequence_number;
        let fragment_starting_num = (fragment_number + 1) as u32;
        let fragments_in_submessage = 1;
        let fragment_size = data_max_size_serialized as u16;
        let data_size = self.data_value.len() as u32;

        let start = fragment_number * data_max_size_serialized;
        let end = core::cmp::min(
            (fragment_number + 1) * data_max_size_serialized,
            self.data_value.len(),
        );

        let serialized_payload = &self.data_value.as_ref()[start..end];

        DataFragSubmessageWrite::new(
            inline_qos_flag,
            non_standard_payload_flag,
            key_flag,
            reader_id,
            writer_id,
            writer_sn,
            fragment_starting_num,
            fragments_in_submessage,
            fragment_size,
            data_size,
            self.inline_qos,
            serialized_payload,
        )
    }
}
