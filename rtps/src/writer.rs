use std::sync::{Arc, Mutex};
use crate::behavior::StatefulWriter;

use rust_dds_interface::protocol::{ProtocolEntity, ProtocolWriter};
use rust_dds_interface::types::{ReturnCode, InstanceHandle};

pub struct Writer {
    writer: Arc<Mutex<StatefulWriter>>,
}

impl Writer {
    pub fn new(writer: Arc<Mutex<StatefulWriter>>) -> Self {
        Self {
            writer
        }
    }
}


impl ProtocolEntity for Writer {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        todo!()
    }
}

impl ProtocolWriter for Writer {
    fn write(&mut self, _instance_handle: InstanceHandle, _data: rust_dds_interface::types::Data, _timestamp: rust_dds_interface::types::Time) -> ReturnCode<()> {
        todo!()
    }

    fn dispose(&self, _instance_handle: InstanceHandle, _timestamp: rust_dds_interface::types::Time) -> ReturnCode<()> {
        todo!()
    }

    fn unregister(&self, _instance_handle: InstanceHandle, _timestamp: rust_dds_interface::types::Time) -> ReturnCode<()> {
        todo!()
    }

    fn register(&self, _instance_handle: InstanceHandle, _timestamp: rust_dds_interface::types::Time) -> ReturnCode<Option<InstanceHandle>> {
        todo!()
    }

    fn lookup_instance(&self, _instance_handle: InstanceHandle) -> Option<InstanceHandle> {
        todo!()
    }
}