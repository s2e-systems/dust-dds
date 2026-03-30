use super::writer_proxy::RtpsWriterProxy;
use crate::transport::types::{Guid, ReliabilityKind, WriterProxy};
use alloc::vec::Vec;

pub struct RtpsStatefulReader {
    guid: Guid,
    matched_writers: Vec<RtpsWriterProxy>,
    reliability: ReliabilityKind,
}

impl RtpsStatefulReader {
    pub fn new(guid: Guid, reliability: ReliabilityKind) -> Self {
        Self {
            guid,
            matched_writers: Vec::new(),
            reliability,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn add_matched_writer(&mut self, writer_proxy: &WriterProxy) {
        let rtps_writer_proxy = RtpsWriterProxy::new(
            writer_proxy.remote_writer_guid,
            &writer_proxy.unicast_locator_list,
            &writer_proxy.multicast_locator_list,
            writer_proxy.remote_group_entity_id,
            writer_proxy.reliability_kind,
        );
        if let Some(wp) = self
            .matched_writers
            .iter_mut()
            .find(|wp| wp.remote_writer_guid() == writer_proxy.remote_writer_guid)
        {
            *wp = rtps_writer_proxy;
        } else {
            self.matched_writers.push(rtps_writer_proxy);
        }
    }

    pub fn delete_matched_writer(&mut self, writer_guid: Guid) {
        self.matched_writers
            .retain(|writer_proxy| writer_proxy.remote_writer_guid() != writer_guid)
    }

    pub fn matched_writer_lookup(&mut self, a_writer_guid: Guid) -> Option<&mut RtpsWriterProxy> {
        self.matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == a_writer_guid)
    }

    pub fn reliability(&self) -> ReliabilityKind {
        self.reliability
    }
}

// The methods in this impl block are not defined by the standard
impl RtpsStatefulReader {
    pub fn is_historical_data_received(&self) -> bool {
        !self
            .matched_writers
            .iter()
            .any(|p| !p.is_historical_data_received())
    }
}
