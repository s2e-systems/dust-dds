use std::collections::HashMap;

use crate::rtps::types::{ReliabilityKind, GUID, GuidPrefix };
use crate::rtps::messages::RtpsSubmessage;

use crate::rtps::behavior::types::Duration;
use crate::rtps::behavior::{Reader, WriterProxy};
use crate::rtps::behavior::endpoint_traits::{DestinedMessages, CacheChangeReceiver, AcknowldegmentSender};
use super::best_effort_writer_proxy::BestEffortWriterProxy;
use super::reliable_writer_proxy::ReliableWriterProxy;

use crate::types::TopicKind;
use crate::rtps::structure::HistoryCache;

enum WriterProxyFlavor{
    BestEffort(BestEffortWriterProxy),
    Reliable(ReliableWriterProxy),
}

impl std::ops::Deref for WriterProxyFlavor {
    type Target = WriterProxy;

    fn deref(&self) -> &Self::Target {
        match self {
            WriterProxyFlavor::BestEffort(wp) => wp,
            WriterProxyFlavor::Reliable(wp) => wp,
        }
    }
}

pub struct StatefulReader {
    pub reader: Reader,
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

            let reader = Reader::new(guid, topic_kind, reliability_level, reader_cache, expects_inline_qos);
            Self {
                reader,
                heartbeat_response_delay,
                matched_writers: HashMap::new()
            }
    }
    
    pub fn matched_writer_add(&mut self, a_writer_proxy: WriterProxy) {
        let remote_writer_guid = a_writer_proxy.remote_writer_guid.clone();
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
            Some(writer_proxy_flavor) => Some(writer_proxy_flavor),
            None => None,
        }
    }
}

impl CacheChangeReceiver for StatefulReader {
    fn try_process_message(&mut self, source_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        for (_writer_guid, writer_proxy) in self.matched_writers.iter_mut() {
            match writer_proxy {
                WriterProxyFlavor::BestEffort(best_effort_writer_proxy) => best_effort_writer_proxy.try_process_message(source_guid_prefix, submessage, &mut self.reader.reader_cache),
                WriterProxyFlavor::Reliable(reliable_writer_proxy) => reliable_writer_proxy.try_process_message(source_guid_prefix, submessage, &mut self.reader.reader_cache),
            }
        }
    }
}

impl AcknowldegmentSender for StatefulReader {
    fn produce_messages(&mut self) -> Vec<DestinedMessages> {
        let mut output = Vec::new();
        for (_writer_guid, writer_proxy) in self.matched_writers.iter_mut(){
            match writer_proxy {
                WriterProxyFlavor::BestEffort(_) => (),
                WriterProxyFlavor::Reliable(reliable_writer_proxy) => {
                    let messages = reliable_writer_proxy.produce_messages(self.reader.endpoint.entity.guid.entity_id(), self.heartbeat_response_delay);
                    output.push( {
                        DestinedMessages::MultiDestination{
                            unicast_locator_list: reliable_writer_proxy.unicast_locator_list.clone(),
                            multicast_locator_list: reliable_writer_proxy.multicast_locator_list.clone(),
                            messages
                        }
                    })
                }
            }
        }
        output
    }
}