use rust_dds_interface::types::InstanceHandle;

pub struct DiscoveredWriterData {
    id: u32,
}

impl DiscoveredWriterData {
    pub fn new() -> Self {
        Self {
            id: 0
        }
    }

    pub fn key(&self) -> InstanceHandle {
        [0;16]
    }

    pub fn data(&self) -> Vec<u8> {
        self.id.to_be_bytes().into()
    }

    pub fn from_key_data(key: InstanceHandle, data: &[u8]) -> Self {
        todo!()
    }
}