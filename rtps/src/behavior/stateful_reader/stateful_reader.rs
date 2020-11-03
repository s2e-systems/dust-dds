use std::collections::HashMap;

use crate::structure::{HistoryCache, HistoryCacheResourceLimits, RtpsEndpoint, RtpsEntity, CacheChange};
use crate::types::{Locator, ReliabilityKind, TopicKind, GUID, GuidPrefix };
use crate::messages::RtpsSubmessage;
use crate::behavior::types::Duration;

use crate::behavior::WriterProxy;
use super::best_effort_writer_proxy::BestEffortWriterProxy;
use super::reliable_writer_proxy::ReliableWriterProxy;

use rust_dds_interface::protocol::{ProtocolEntity, ProtocolReader};
use rust_dds_interface::types::{InstanceHandle, ReturnCode};

pub enum WriterProxyFlavor{
    BestEffort(BestEffortWriterProxy),
    Reliable(ReliableWriterProxy),
}

pub trait StatefulReaderListener {
    fn on_add_change(&self, cc: &CacheChange) -> (){}
}

struct NoOpStatefulReaderListener;
impl StatefulReaderListener for NoOpStatefulReaderListener {}

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

    reader_cache: HistoryCache,

    // Fields
    matched_writers: HashMap<GUID, WriterProxyFlavor>,

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
        resource_limits: HistoryCacheResourceLimits
        ) -> Self {
            Self {
                guid,
                topic_kind,
                reliability_level,
                expects_inline_qos,
                heartbeat_response_delay,       
                reader_cache: HistoryCache::new(resource_limits),
                matched_writers: HashMap::new(),
                listener: Box::new(NoOpStatefulReaderListener),
            }
    }

    pub fn try_process_message(&mut self, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
        for (_writer_guid, writer_proxy) in self.matched_writers.iter_mut() {
            match writer_proxy {
                WriterProxyFlavor::BestEffort(best_effort_writer_proxy) => best_effort_writer_proxy.try_process_message(src_guid_prefix, submessage, &mut self.reader_cache, self.listener.as_ref()),
                WriterProxyFlavor::Reliable(reliable_writer_proxy) => reliable_writer_proxy.try_process_message(src_guid_prefix, submessage, &mut self.reader_cache),
            }
        }
    }
    
    pub fn matched_writer_add(&mut self, a_writer_proxy: WriterProxy) {
        let remote_writer_guid = a_writer_proxy.remote_writer_guid().clone();
        let writer_proxy = match self.reliability_level {
            ReliabilityKind::Reliable => WriterProxyFlavor::Reliable(ReliableWriterProxy::new(a_writer_proxy)),
            ReliabilityKind::BestEffort => WriterProxyFlavor::BestEffort(BestEffortWriterProxy::new(a_writer_proxy)),
        };
        
        self.matched_writers.insert(remote_writer_guid, writer_proxy);
    }

    pub fn matched_writer_remove(&mut self, writer_proxy_guid: &GUID) {
        self.matched_writers.remove(writer_proxy_guid);
    }

    pub fn matched_writers(&mut self) -> &mut HashMap<GUID, WriterProxyFlavor> {
        &mut self.matched_writers
    }

    pub fn reader_cache(&self) -> &HistoryCache {
        &self.reader_cache
    }

    pub fn heartbeat_response_delay(&self) -> Duration {
        self.heartbeat_response_delay
    }

    pub fn guid(&self) -> &GUID {
        &self.guid
    }
}

impl ProtocolEntity for StatefulReader {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
    }
}

impl ProtocolReader for StatefulReader {}

impl RtpsEntity for StatefulReader {
    fn guid(&self) -> GUID {
        self.guid
    }
}

impl RtpsEndpoint for StatefulReader {
    fn unicast_locator_list(&self) -> Vec<Locator> {
        todo!()
    }

    fn multicast_locator_list(&self) -> Vec<Locator> {
        todo!()
    }

    fn reliability_level(&self) -> ReliabilityKind {
        todo!()
    }

    fn topic_kind(&self) -> &TopicKind {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// impl RtpsCommunication for StatefulReader {
//     fn try_push_message(&mut self, src_locator: Locator, src_guid_prefix: GuidPrefix, submessage: &mut Option<RtpsSubmessage>) {
//         
//     }
// }

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;


}
