use std::collections::HashMap;

use crate::types::{ReliabilityKind, GUID, GuidPrefix};
use crate::messages::RtpsSubmessage;
use crate::behavior::RtpsWriter;
use crate::behavior::types::Duration;
use crate::behavior::endpoint_traits::{DestinedMessages, AcknowldegmentReceiver, CacheChangeSender};

use super::reader_proxy::ReaderProxy;
use super::reliable_reader_proxy::ReliableReaderProxy;
use super::best_effort_reader_proxy::BestEffortReaderProxy;
use rust_dds_api::types::TopicKind;
use crate::structure::HistoryCache;

enum ReaderProxyFlavor {
    BestEffort(BestEffortReaderProxy),
    Reliable(ReliableReaderProxy),
}

impl std::ops::Deref for ReaderProxyFlavor {
    type Target = ReaderProxy;

    fn deref(&self) -> &Self::Target {
        match self {
            ReaderProxyFlavor::BestEffort(rp) => rp,
            ReaderProxyFlavor::Reliable(rp) => rp,
        }
    }
}

pub struct StatefulWriter {
    pub writer: RtpsWriter,
    pub heartbeat_period: Duration,
    pub nack_response_delay: Duration,
    pub nack_suppression_duration: Duration,
    matched_readers: HashMap<GUID, ReaderProxyFlavor>,
}

impl StatefulWriter {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        push_mode: bool,
        writer_cache: HistoryCache,
        data_max_sized_serialized: Option<i32>,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
    ) -> Self {
        
        let writer = RtpsWriter::new(guid, topic_kind, reliability_level, push_mode, writer_cache, data_max_sized_serialized);
            Self {
                writer,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
                matched_readers: HashMap::new()
        }
    }
    
    pub fn matched_reader_add(&mut self, a_reader_proxy: ReaderProxy) {
        let remote_reader_guid = a_reader_proxy.remote_reader_guid;
        let reader_proxy = match self.writer.endpoint.reliability_level {
            ReliabilityKind::Reliable => ReaderProxyFlavor::Reliable(ReliableReaderProxy::new(a_reader_proxy)),
            ReliabilityKind::BestEffort => ReaderProxyFlavor::BestEffort(BestEffortReaderProxy::new(a_reader_proxy)),
        };
        self.matched_readers.insert(remote_reader_guid, reader_proxy);
    }

    pub fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID) {
        self.matched_readers.remove(reader_proxy_guid);
    }

    pub fn matched_reader_lookup(&self, a_reader_guid: GUID) -> Option<&ReaderProxy> {
        match self.matched_readers.get(&a_reader_guid) {
            Some(rp) => Some(rp),
            None => None,
        }
    }

    pub fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

impl CacheChangeSender for StatefulWriter {
    fn produce_messages(&mut self) -> Vec<DestinedMessages> {
        let mut output = Vec::new();
        for (_reader_guid, reader_proxy) in self.matched_readers.iter_mut() {
            let messages = match reader_proxy {
                ReaderProxyFlavor::Reliable(reliable_reader_proxy) => reliable_reader_proxy.produce_messages(&self.writer.writer_cache, self.writer.endpoint.entity.guid.entity_id(), self.writer.last_change_sequence_number, self.heartbeat_period, self.nack_response_delay),
                ReaderProxyFlavor::BestEffort(best_effort_reader_proxy) => best_effort_reader_proxy.produce_messages(&self.writer.writer_cache, self.writer.endpoint.entity.guid.entity_id(), self.writer.last_change_sequence_number),
            };

            if !messages.is_empty() {
                output.push(DestinedMessages::MultiDestination{
                    unicast_locator_list: reader_proxy.unicast_locator_list.clone(),
                    multicast_locator_list: reader_proxy.multicast_locator_list.clone(),
                    messages,
                });   
            }
        }
        output
    }
}

impl AcknowldegmentReceiver for StatefulWriter {
    fn try_process_message(&mut self, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        for (_reader_guid, reader_proxy) in self.matched_readers.iter_mut(){
            match reader_proxy {
                ReaderProxyFlavor::Reliable(reliable_reader_proxy) => reliable_reader_proxy.try_process_message(src_guid_prefix, submessage),
                ReaderProxyFlavor::BestEffort(_) => ()
            }
        }
    }
}