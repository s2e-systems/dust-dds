use crate::{
    implementation::data_representation_inline_qos::{
        parameter_id_values::PID_STATUS_INFO,
        types::{StatusInfo, STATUS_INFO_DISPOSED_FLAG, STATUS_INFO_UNREGISTERED_FLAG},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        time::Time,
    },
    subscription::sample_info::SampleStateKind,
};

use super::{
    history_cache::RtpsParameter,
    messages::{submessages::DataSubmessage, types::ParameterId},
    types::{ChangeKind, Guid, GuidPrefix, SequenceNumber},
};

#[derive(Debug, Clone)]

pub struct RtpsReaderCacheChange {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    data: Vec<u8>,
    _inline_qos: Vec<RtpsParameter>,
    source_timestamp: Option<Time>,
    sample_state: SampleStateKind,
}

impl PartialEq for RtpsReaderCacheChange {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.writer_guid == other.writer_guid
            && self.sequence_number == other.sequence_number
    }
}

impl RtpsReaderCacheChange {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kind: ChangeKind,
        writer_guid: Guid,
        sequence_number: SequenceNumber,
        data_value: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        source_timestamp: Option<Time>,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            sequence_number,
            data: data_value,
            _inline_qos: inline_qos,
            source_timestamp,
            sample_state: SampleStateKind::NotRead,
        }
    }

    pub fn kind(&self) -> ChangeKind {
        self.kind
    }

    pub fn writer_guid(&self) -> Guid {
        self.writer_guid
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

    pub fn mark_read(&mut self) {
        self.sample_state = SampleStateKind::Read;
    }

    pub fn try_from_data_submessage(
        data: &DataSubmessage,
        source_timestamp: Option<Time>,
        source_guid_prefix: GuidPrefix,
    ) -> DdsResult<Self> {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value.into());

        let sequence_number = data.writer_sn.value;
        let data_value = data.serialized_payload.value.to_vec();

        let inline_qos: Vec<RtpsParameter> = data
            .inline_qos
            .parameter
            .iter()
            .map(|p| RtpsParameter::new(ParameterId(p.parameter_id), p.value.to_vec()))
            .collect();

        let kind = match (data.data_flag, data.key_flag) {
            (true, false) => Ok(ChangeKind::Alive),
            (false, true) => {
                if let Some(p) = inline_qos
                    .iter()
                    .find(|&x| x.parameter_id() == ParameterId(PID_STATUS_INFO))
                {
                    let mut deserializer =
                        cdr::Deserializer::<_, _, cdr::LittleEndian>::new(p.value(), cdr::Infinite);
                    let status_info: StatusInfo =
                        serde::Deserialize::deserialize(&mut deserializer).unwrap();
                    match status_info {
                        STATUS_INFO_DISPOSED_FLAG => Ok(ChangeKind::NotAliveDisposed),
                        STATUS_INFO_UNREGISTERED_FLAG => Ok(ChangeKind::NotAliveUnregistered),
                        _ => Err(DdsError::PreconditionNotMet(
                            "Unknown status info value".to_string(),
                        )),
                    }
                } else {
                    Err(DdsError::PreconditionNotMet(
                        "Missing mandatory StatusInfo parameter".to_string(),
                    ))
                }
            }
            _ => Err(DdsError::PreconditionNotMet(
                "Invalid data submessage data and key flag combination".to_string(),
            )),
        }?;

        Ok(RtpsReaderCacheChange::new(
            kind,
            writer_guid,
            sequence_number,
            data_value,
            inline_qos,
            source_timestamp,
        ))
    }
}
