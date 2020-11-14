use std::sync::Mutex;
use std::collections::HashMap;

use crate::structure::{RtpsEndpoint, RtpsEntity};
use crate::types::{ReliabilityKind, GUID, GuidPrefix };
use crate::messages::RtpsSubmessage;
use crate::behavior::types::Duration;

use crate::behavior::{WriterProxy, DestinedMessages};
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
    // From Entity base class
    guid: GUID,
    // entity: Entity,

    // From Endpoint base class:
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,

    // All communication to this reader is done by the writer proxies
    // so these fields are unnecessary
    // unicast_locator_list: Vec<Locator>,
    // multicast_locator_list: Vec<Locator>,

    // From Reader base class:
    expects_inline_qos: bool,
    heartbeat_response_delay: Duration,

    reader_cache: Mutex<HistoryCache>,

    // Fields
    matched_writers: Mutex<HashMap<GUID, WriterProxyFlavor>>,

    // Additional fields:
    listener: Box<dyn StatefulReaderListener>,
}

impl StatefulReader {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        expects_inline_qos: bool,
        heartbeat_response_delay: Duration,
        reader_cache: HistoryCache,
        listener: impl StatefulReaderListener,
        ) -> Self {
            Self {
                guid,
                topic_kind,
                reliability_level,
                expects_inline_qos,
                heartbeat_response_delay,       
                reader_cache: Mutex::new(reader_cache),
                matched_writers: Mutex::new(HashMap::new()),
                listener: Box::new(listener),
            }
    }

    pub fn try_process_message(&self, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        let mut matched_writers = self.matched_writers.lock().unwrap();
        for (_writer_guid, writer_proxy) in matched_writers.iter_mut() {
            match writer_proxy {
                WriterProxyFlavor::BestEffort(best_effort_writer_proxy) => best_effort_writer_proxy.try_process_message(src_guid_prefix, submessage, &mut self.reader_cache.lock().unwrap(), self.listener.as_ref()),
                WriterProxyFlavor::Reliable(reliable_writer_proxy) => reliable_writer_proxy.try_process_message(src_guid_prefix, submessage, &mut self.reader_cache.lock().unwrap(), self.listener.as_ref()),
            }
        }
    }

    pub fn produce_messages(&self) -> Vec<DestinedMessages> {
        let mut matched_writers = self.matched_writers.lock().unwrap();
        let mut output = Vec::new();
        for (_writer_guid, writer_proxy) in matched_writers.iter_mut(){
            match writer_proxy {
                WriterProxyFlavor::BestEffort(_) => (),
                WriterProxyFlavor::Reliable(reliable_writer_proxy) => {
                    let messages = reliable_writer_proxy.produce_messages(self.guid.entity_id(), self.heartbeat_response_delay);
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
    
    pub fn matched_writer_add(&self, a_writer_proxy: WriterProxy) {
        let remote_writer_guid = a_writer_proxy.remote_writer_guid().clone();
        let writer_proxy = match self.reliability_level {
            ReliabilityKind::Reliable => WriterProxyFlavor::Reliable(ReliableWriterProxy::new(a_writer_proxy)),
            ReliabilityKind::BestEffort => WriterProxyFlavor::BestEffort(BestEffortWriterProxy::new(a_writer_proxy)),
        };
        
        self.matched_writers.lock().unwrap().insert(remote_writer_guid, writer_proxy);
    }

    pub fn matched_writer_remove(&self, writer_proxy_guid: &GUID) {
        self.matched_writers.lock().unwrap().remove(writer_proxy_guid);
    }

    pub fn reader_cache(&self) -> &Mutex<HistoryCache> {
        &self.reader_cache
    }

    pub fn heartbeat_response_delay(&self) -> Duration {
        self.heartbeat_response_delay
    }

    pub fn guid(&self) -> &GUID {
        &self.guid
    }
}

impl RtpsEntity for StatefulReader {
    fn guid(&self) -> GUID {
        self.guid
    }
}

impl RtpsEndpoint for StatefulReader {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;


}
