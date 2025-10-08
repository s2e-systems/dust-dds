use super::error::RtpsError;
use crate::xtypes::deserialize::XTypesDeserialize;
use crate::{
    rtps_messages::{
        self,
        submessage_elements::{Parameter, ParameterList},
        submessages::data::DataSubmessage,
        types::ParameterId,
    },
    transport::types::{CacheChange, ChangeKind, EntityId, Guid, GuidPrefix},
};
use alloc::{sync::Arc, vec::Vec};

pub const PID_KEY_HASH: ParameterId = 0x0070;
pub const PID_STATUS_INFO: ParameterId = 0x0071;

#[derive(Clone, Copy, PartialEq, Eq, XTypesDeserialize, Debug)]
struct StatusInfo(pub [u8; 4]);
const STATUS_INFO_DISPOSED: StatusInfo = StatusInfo([0, 0, 0, 0b00000001]);
const STATUS_INFO_UNREGISTERED: StatusInfo = StatusInfo([0, 0, 0, 0b0000010]);
const STATUS_INFO_DISPOSED_UNREGISTERED: StatusInfo = StatusInfo([0, 0, 0, 0b00000011]);
const STATUS_INFO_FILTERED: StatusInfo = StatusInfo([0, 0, 0, 0b0000100]);

impl CacheChange {
    pub fn as_data_submessage(&self, reader_id: EntityId, writer_id: EntityId) -> DataSubmessage {
        let (data_flag, key_flag) = match self.kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => (true, false),
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (false, true),
        };

        let mut parameters = Vec::with_capacity(2);
        match self.kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => (),
            ChangeKind::NotAliveDisposed => parameters.push(Parameter::new(
                PID_STATUS_INFO,
                Arc::from(STATUS_INFO_DISPOSED.0),
            )),
            ChangeKind::NotAliveUnregistered => parameters.push(Parameter::new(
                PID_STATUS_INFO,
                Arc::from(STATUS_INFO_UNREGISTERED.0),
            )),
            ChangeKind::NotAliveDisposedUnregistered => parameters.push(Parameter::new(
                PID_STATUS_INFO,
                Arc::from(STATUS_INFO_DISPOSED_UNREGISTERED.0),
            )),
        }

        if let Some(i) = self.instance_handle {
            parameters.push(Parameter::new(PID_KEY_HASH, Arc::from(i)));
        }
        let parameter_list = ParameterList::new(parameters);

        DataSubmessage::new(
            true,
            data_flag,
            key_flag,
            false,
            reader_id,
            writer_id,
            self.sequence_number,
            parameter_list,
            self.data_value.clone().into(),
        )
    }

    pub fn try_from_data_submessage(
        data_submessage: &DataSubmessage,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<rtps_messages::types::Time>,
    ) -> Result<Self, RtpsError> {
        let kind = match data_submessage
            .inline_qos()
            .parameter()
            .iter()
            .find(|&x| x.parameter_id() == PID_STATUS_INFO)
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

        let instance_handle = match data_submessage
            .inline_qos()
            .parameter()
            .iter()
            .find(|&x| x.parameter_id() == PID_KEY_HASH)
        {
            Some(p) => <[u8; 16]>::try_from(p.value()).ok(),

            None => None,
        };

        Ok(CacheChange {
            kind,
            writer_guid: Guid::new(source_guid_prefix, data_submessage.writer_id()),
            source_timestamp: source_timestamp.map(Into::into),
            instance_handle,
            sequence_number: data_submessage.writer_sn(),
            data_value: data_submessage.serialized_payload().clone().into(),
        })
    }
}
