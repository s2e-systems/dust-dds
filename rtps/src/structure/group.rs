use std::sync::{Arc, Mutex};

use rust_dds_interface::types::{ReturnCode, InstanceHandle, TopicKind};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolPublisher, ProtocolSubscriber, ProtocolWriter, ProtocolReader};
use rust_dds_interface::qos::{DataWriterQos, DataReaderQos};
use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;

use crate::types::{GUID, EntityKind, EntityId, ReliabilityKind};
use crate::structure::{RtpsEndpoint, RtpsEntity, };
use crate::structure::HistoryCacheResourceLimits;
use crate::behavior::{StatefulReader, StatefulWriter};
use crate::behavior::types::Duration;

pub struct RtpsGroup {
    guid: GUID,
    endpoint_counter: usize,
    endpoints: Vec<Arc<Mutex<dyn RtpsEndpoint>>>,
}

impl RtpsGroup {
    pub fn new(guid: GUID) -> Self {
        Self {
            guid,
            endpoint_counter: 0,
            endpoints: Vec::new(),
        }
    }

    pub fn mut_endpoints(&mut self) -> &mut Vec<Arc<Mutex<dyn RtpsEndpoint>>> {
        &mut self.endpoints
    }

    pub fn endpoints(&self) -> &[Arc<Mutex<dyn RtpsEndpoint>>] {
        self.endpoints.as_slice()
    }
}

impl<'a> IntoIterator for &'a RtpsGroup {
    type Item = &'a Arc<Mutex<dyn RtpsEndpoint>>;
    type IntoIter = std::slice::Iter<'a, Arc<Mutex<dyn RtpsEndpoint>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.endpoints.iter()
    }
}

impl RtpsEntity for RtpsGroup {
    fn guid(&self) -> GUID {
        self.guid
    }
}

impl ProtocolEntity for RtpsGroup {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
    }
}

impl ProtocolPublisher for RtpsGroup {
    
}

impl ProtocolSubscriber for RtpsGroup {
    fn create_reader(&mut self, topic_kind: TopicKind, data_reader_qos: &DataReaderQos) -> Box<dyn ProtocolReader> {
        todo!()
    }
}