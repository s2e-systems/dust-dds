use std::collections::HashMap;

use crate::types::{ReliabilityKind, GUID, GuidPrefix };
use crate::messages::RtpsSubmessage;

use crate::behavior::types::Duration;
use crate::behavior::{RtpsReader, WriterProxy};
use crate::behavior::endpoint_traits::DestinedMessages;
use super::stateful_reader_listener::StatefulReaderListener;
use super::best_effort_writer_proxy::BestEffortWriterProxy;
use super::reliable_writer_proxy::ReliableWriterProxy;

use rust_dds_interface::types::TopicKind;
use rust_dds_interface::history_cache::HistoryCache;

enum WriterProxyFlavor{
    BestEffort(BestEffortWriterProxy),
    Reliable(ReliableWriterProxy),
}

pub struct StatefulReader {
    pub reader: RtpsReader,
    pub heartbeat_response_delay: Duration,
    matched_writers: HashMap<GUID, WriterProxyFlavor>,
}

impl StatefulReader {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        reader_cache: HistoryCache,
        expects_inline_qos: bool,
        heartbeat_response_delay: Duration
        ) -> Self {

            let reader = RtpsReader::new(guid, topic_kind, reliability_level, reader_cache, expects_inline_qos);
            Self {
                reader,
                heartbeat_response_delay,
                matched_writers: HashMap::new()
            }
    }

    pub fn try_process_message(&mut self, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>, reader_cache: &mut HistoryCache, listener: &dyn StatefulReaderListener) {
        for (_writer_guid, writer_proxy) in self.matched_writers.iter_mut() {
            match writer_proxy {
                WriterProxyFlavor::BestEffort(best_effort_writer_proxy) => best_effort_writer_proxy.try_process_message(src_guid_prefix, submessage, reader_cache, listener),
                WriterProxyFlavor::Reliable(reliable_writer_proxy) => reliable_writer_proxy.try_process_message(src_guid_prefix, submessage, reader_cache, listener),
            }
        }
    }

    pub fn produce_messages(&mut self) -> Vec<DestinedMessages> {
        let mut output = Vec::new();
        for (_writer_guid, writer_proxy) in self.matched_writers.iter_mut(){
            match writer_proxy {
                WriterProxyFlavor::BestEffort(_) => (),
                WriterProxyFlavor::Reliable(reliable_writer_proxy) => {
                    let messages = reliable_writer_proxy.produce_messages(self.reader.endpoint.entity.guid.entity_id(), self.heartbeat_response_delay);
                    output.push( {
                        DestinedMessages::MultiDestination{
                            unicast_locator_list: reliable_writer_proxy.unicast_locator_list().clone(),
                            multicast_locator_list: reliable_writer_proxy.multicast_locator_list().clone(),
                            messages
                        }
                    })
                }
            }
        }
        output

    }
    
    pub fn matched_writer_add(&mut self, a_writer_proxy: WriterProxy) {
        let remote_writer_guid = a_writer_proxy.remote_writer_guid().clone();
        let writer_proxy = match self.reader.endpoint.reliability_level {
            ReliabilityKind::Reliable => WriterProxyFlavor::Reliable(ReliableWriterProxy::new(a_writer_proxy)),
            ReliabilityKind::BestEffort => WriterProxyFlavor::BestEffort(BestEffortWriterProxy::new(a_writer_proxy)),
        };
        
        self.matched_writers.insert(remote_writer_guid, writer_proxy);
    }

    pub fn matched_writer_remove(&mut self, writer_proxy_guid: &GUID) {
        self.matched_writers.remove(writer_proxy_guid);
    }

    pub fn matched_writer_lookup(&self, a_writer_guid: GUID) -> Option<&WriterProxy> {
        match self.matched_writers.get(&a_writer_guid) {
            Some(writer_proxy_flavor) => match writer_proxy_flavor {
                WriterProxyFlavor::BestEffort(wp) => Some(wp),
                WriterProxyFlavor::Reliable(wp) => Some(wp),
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;


}
