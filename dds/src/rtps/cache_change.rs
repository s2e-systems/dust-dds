use crate::transport::{types::ChangeKind, writer::RtpsCacheChange};

use super::{messages::submessages::data::DataSubmessage, types::EntityId};

impl RtpsCacheChange {
    pub fn as_data_submessage(&self, reader_id: EntityId, writer_id: EntityId) -> DataSubmessage {
        let (data_flag, key_flag) = match self.kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => (true, false),
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => (false, true),
        };

        DataSubmessage::new(
            true,
            data_flag,
            key_flag,
            false,
            reader_id,
            writer_id,
            self.sequence_number,
            self.inline_qos.clone(),
            self.data_value.clone(),
        )
    }
}
