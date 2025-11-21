use crate::{
    rtps::history_cache::HistoryCache,
    rtps_messages::{self, submessages::data::DataSubmessage},
    transport::types::{CacheChange, ENTITYID_UNKNOWN, Guid, GuidPrefix},
};
use alloc::boxed::Box;

pub struct RtpsStatelessReader {
    guid: Guid,
    history_cache: Box<dyn HistoryCache + Send + Sync>,
}

impl RtpsStatelessReader {
    pub fn new(guid: Guid, history_cache: Box<dyn HistoryCache + Send + Sync>) -> Self {
        Self {
            guid,
            history_cache,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub async fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<rtps_messages::types::Time>,
    ) {
        if data_submessage.reader_id() == ENTITYID_UNKNOWN
            || data_submessage.reader_id() == self.guid.entity_id()
        {
            if let Ok(change) = CacheChange::try_from_data_submessage(
                data_submessage,
                source_guid_prefix,
                source_timestamp,
            ) {
                // Stateless reader behavior. We add the change if the data is correct. No error is printed
                // because all readers would get changes marked with ENTITYID_UNKNOWN
                self.history_cache.add_change(change).await;
            }
        }
    }
}
