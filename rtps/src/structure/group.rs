use std::sync::{Weak, Arc};

use super::stateful_writer::StatefulWriter;
use super::stateful_reader::StatefulReader;

use rust_dds_interface::protocol::{ProtocolEntity, ProtocolGroup};

pub struct Group {
    readers: Vec<Arc<StatefulReader>>,
    writers: Vec<Arc<StatefulWriter>>
}

impl Group {
    pub fn new() -> Self {
        Self {
            readers: vec![], 
            writers: vec![]
        }
    }
}

impl ProtocolEntity for Group{
    fn enable(&self) -> rust_dds_interface::types::ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> rust_dds_interface::types::InstanceHandle {
        todo!()
    }
}
impl ProtocolGroup for Group{
    fn create_writer(&self) -> Weak<dyn rust_dds_interface::protocol::ProtocolWriter> {
        todo!()
    }

    fn create_reader(&self) -> Weak<dyn rust_dds_interface::protocol::ProtocolReader> {
        todo!()
    }
}