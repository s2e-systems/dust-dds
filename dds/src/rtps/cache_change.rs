use std::sync::Arc;

use crate::{
    implementation::data_representation_inline_qos::{
        parameter_id_values::{PID_KEY_HASH, PID_STATUS_INFO},
        types::{
            StatusInfo, STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED,
            STATUS_INFO_FILTERED, STATUS_INFO_UNREGISTERED,
        },
    },
    transport::{
        history_cache::CacheChange,
        types::{ChangeKind, EntityId, Guid, GuidPrefix},
    },
};

use super::messages::{
    self,
    submessage_elements::{Parameter, ParameterList},
    submessages::data::DataSubmessage,
};

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
        source_timestamp: Option<messages::types::Time>,
    ) -> Result<Self, String> {
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
                        _ => Err(format!(
                            "Received invalid status info parameter with value {:?}",
                            status_info
                        )),
                    }
                } else {
                    Err(format!(
                        "Received invalid status info parameter length. Expected 4, got {:?}",
                        p.length()
                    ))
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
            Some(p) => {
                if let Ok(key) = <[u8; 16]>::try_from(p.value()) {
                    Some(key)
                } else {
                    None
                }
            }
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
