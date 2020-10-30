use std::sync::{Arc, Mutex};

use rust_dds_interface::types::{ReturnCode, InstanceHandle, TopicKind};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolPublisher, ProtocolSubscriber, ProtocolWriter, ProtocolReader};
use rust_dds_interface::qos::{DataWriterQos, DataReaderQos};

use crate::types::{GUID, Locator, GuidPrefix};
use crate::messages::RtpsSubmessage;
use crate::messages::message_sender::RtpsMessageSender;
use crate::structure::{RtpsEndpoint, RtpsEntity, RtpsRun, OutputQueue};



pub struct RtpsGroup {
    guid: GUID,
    sender: RtpsMessageSender,
    endpoints: Vec<Arc<Mutex<dyn RtpsEndpoint>>>,
}

impl RtpsGroup {
    pub fn new(guid: GUID, sender: RtpsMessageSender,) -> Self {
        Self {
            guid,
            sender,
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

impl RtpsRun for RtpsGroup {
    fn run(&mut self) {
        let participant_guid_prefix = self.guid.prefix();
        for endpoint in &mut self.endpoints {
            let queues = endpoint.lock().unwrap().output_queues();
            for queue in queues {
                match queue {
                    OutputQueue::SingleDestination { locator, message_queue } => {
                        self.sender.send(participant_guid_prefix, &locator, message_queue.into())
                    }
                    OutputQueue::MultiDestination { unicast_locator_list, multicast_locator_list, message_queue } => {
                        for locator in unicast_locator_list {
                            self.sender.send(participant_guid_prefix, &locator, message_queue.into());
                            break; // Take only first element for now
                        }
                        for _locator in multicast_locator_list {
                            break; // Take only first element for now
                        }
                    }
                }
            }
        }
        
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
    fn create_writer(&mut self, _topic_kind: TopicKind, _data_writer_qos: &DataWriterQos) -> Arc<Mutex<dyn ProtocolWriter>> {
        todo!()
    }
}

impl ProtocolSubscriber for RtpsGroup {
    fn create_reader(&mut self, _topic_kind: TopicKind, _data_reader_qos: &DataReaderQos) -> Arc<Mutex<dyn ProtocolReader>> {
        todo!()
    }
}