use std::collections::{HashMap, VecDeque, };
use std::sync::{RwLock, RwLockReadGuard};

use crate::structure::HistoryCache;
use crate::types::{Locator, ReliabilityKind, TopicKind, GUID, GuidPrefix };
use crate::messages::RtpsSubmessage;
use crate::messages::message_receiver::Receiver;
use crate::messages::message_sender::Sender;
use crate::behavior::types::Duration;

use super::WriterProxy;
use crate::behavior::best_effort_writer_proxy::BestEffortWriterProxy;
use crate::behavior::reliable_writer_proxy::ReliableWriterProxy;

use rust_dds_interface::protocol::{ProtocolEntity, ProtocolReader, ProtocolEndpoint};
use rust_dds_interface::qos::DataReaderQos;
use rust_dds_interface::types::{InstanceHandle, ReturnCode};

enum WriterProxyType {
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

    reader_cache: HistoryCache,

    // Fields
    matched_writers: RwLock<HashMap<GUID, WriterProxyType>>,
}

impl StatefulReader {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reader_qos: &DataReaderQos,
        ) -> Self {
            
        let expects_inline_qos = false;
        let heartbeat_response_delay = Duration::from_millis(200);
        
        Self {
            guid,
            topic_kind,
            reliability_level: reader_qos.reliability.kind.into(),
            expects_inline_qos,
            heartbeat_response_delay,       
            reader_cache: HistoryCache::new(&reader_qos.resource_limits),
            matched_writers: RwLock::new(HashMap::new()),
        }
    }

    pub fn matched_writer_add(&self, a_writer_proxy: WriterProxy) {
        todo!()
        // self.matched_writers.write().unwrap().insert(a_writer_proxy.remote_writer_guid, a_writer_proxy);
    }

    pub fn matched_writer_remove(&self, writer_proxy_guid: &GUID) {
        self.matched_writers.write().unwrap().remove(&writer_proxy_guid);
    }
    
    pub fn matched_writers(&self) -> RwLockReadGuard<HashMap<GUID, WriterProxy>> {
        todo!()
        // self.matched_writers.read().unwrap()
    }

    pub fn run(&self) {
        todo!()
        // for (_writer_guid, writer_proxy) in self.matched_writers().iter() {
        //     match self.reliability_level {
        //         ReliabilityKind::BestEffort => BestEfforStatefulReaderBehavior::run(writer_proxy, &self),
        //         ReliabilityKind::Reliable => ReliableStatefulReaderBehavior::run(writer_proxy,  &self),
        //     };
        // }
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

impl Receiver for StatefulReader {
    fn push_receive_message(&self, src_guid_prefix: GuidPrefix, submessage: RtpsSubmessage) {
        let writer_id = match &submessage {
            RtpsSubmessage::Data(data) => data.writer_id(),
            RtpsSubmessage::Gap(gap) => gap.writer_id(),
            RtpsSubmessage::Heartbeat(heartbeat) => 
                if self.reliability_level == ReliabilityKind::Reliable {
                    heartbeat.writer_id()
                } else {
                    panic!("Heartbeat messages not received by best-effort stateful reader")
                },
            _ =>  panic!("Unsupported message received by stateful reader"),
        };
        let writer_guid = GUID::new(src_guid_prefix, writer_id);

        todo!()
        // self.matched_writers().get(&writer_guid).unwrap().received_messages.lock().unwrap().push_back((src_guid_prefix, submessage));
    }
    
    fn pop_receive_message(&self, guid: &GUID) -> Option<(GuidPrefix, RtpsSubmessage)> {
        todo!()
        // self.matched_writers().get(guid).unwrap().received_messages.lock().unwrap().pop_front()
    }

    fn is_submessage_destination(&self, _src_locator: &Locator, src_guid_prefix: &GuidPrefix, submessage: &RtpsSubmessage) -> bool {
        let writer_id = match submessage {
            RtpsSubmessage::Data(data) => data.writer_id(),
            RtpsSubmessage::Gap(gap) => gap.writer_id(),
            RtpsSubmessage::Heartbeat(heartbeat) => if self.reliability_level == ReliabilityKind::Reliable {
                heartbeat.writer_id()
            } else {
                return false
            },
            _ => return false,
        };

        let writer_guid = GUID::new(*src_guid_prefix, writer_id);

        // If the message comes from a matched writer then this should be the destination
        self.matched_writers().get(&writer_guid).is_some()
    }
}

impl Sender for StatefulReader {
    fn push_send_message(&self, _dst_locator: &Locator, dst_guid: &GUID, submessage: RtpsSubmessage) {
        todo!()
        // self.matched_writers().get(dst_guid).unwrap().send_messages.lock().unwrap().push_back(submessage)
    }

    fn pop_send_message(&self) -> Option<(Vec<Locator>, VecDeque<RtpsSubmessage>)> {
        todo!()
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

impl ProtocolEndpoint for StatefulReader {}
impl ProtocolReader for StatefulReader {}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;


}
