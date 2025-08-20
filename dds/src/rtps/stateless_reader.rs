use super::{error::RtpsResult, message_receiver::MessageReceiver};
use crate::{
    rtps_messages::{
        self,
        overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
        submessages::data::DataSubmessage,
    },
    transport::{
        interface::HistoryCache,
        types::{CacheChange, Guid, GuidPrefix, ENTITYID_UNKNOWN},
    },
};
use alloc::boxed::Box;

pub struct RtpsStatelessReader {
    guid: Guid,
    history_cache: Box<dyn HistoryCache>,
}

impl RtpsStatelessReader {
    pub fn new(guid: Guid, history_cache: Box<dyn HistoryCache>) -> Self {
        Self {
            guid,
            history_cache,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    async fn on_data_submessage_received(
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

    pub async fn process_message(&mut self, datagram: &[u8]) -> RtpsResult<()> {
        let rtps_message = RtpsMessageRead::try_from(datagram)?;
        let mut message_receiver = MessageReceiver::new(&rtps_message);

        while let Some(submessage) = message_receiver.next() {
            if let RtpsSubmessageReadKind::Data(data_submessage) = &submessage {
                self.on_data_submessage_received(
                    data_submessage,
                    message_receiver.source_guid_prefix(),
                    message_receiver.source_timestamp(),
                )
                .await;
            }
        }
        Ok(())
    }
}
