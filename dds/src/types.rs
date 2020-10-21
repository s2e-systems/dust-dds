use rust_dds_interface::types::{InstanceHandle, Data, TopicKind};

pub trait DDSType: 'static {
    fn topic_kind() -> TopicKind;
    
    fn instance_handle(&self) -> InstanceHandle;

    fn serialize(&self) -> Data;

    fn deserialize(data: Data) -> Self;
}

 
