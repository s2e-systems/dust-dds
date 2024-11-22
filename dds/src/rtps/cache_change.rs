use std::sync::Arc;

use crate::{
    implementation::data_representation_inline_qos::{
        parameter_id_values::{PID_KEY_HASH, PID_STATUS_INFO},
        types::{
            STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED, STATUS_INFO_UNREGISTERED,
        },
    },
    transport::{cache_change::CacheChange, types::ChangeKind},
};

use super::{
    messages::{
        submessage_elements::{Parameter, ParameterList},
        submessages::data::DataSubmessage,
    },
    types::EntityId,
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
}
