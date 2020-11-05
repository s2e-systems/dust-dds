use std::sync::{Arc, Mutex};
use crate::behavior::stateful_reader::StatefulReader;

use rust_dds_interface::protocol::{ProtocolEntity, ProtocolReader};
use rust_dds_interface::types::{ReturnCode, InstanceHandle};

struct Reader {
    reader: Arc<Mutex<StatefulReader>>,
}

impl Reader {
    pub fn new(reader: Arc<Mutex<StatefulReader>>) -> Self {
        Self {
            reader
        }
    }
}

impl ProtocolEntity for Reader {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        todo!()
    }
}
impl ProtocolReader for Reader {

}