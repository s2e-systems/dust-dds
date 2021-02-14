use crate::types::{GUID, Locator};

pub struct ReaderProxy {
    pub remote_reader_guid: GUID,
    pub expects_inline_qos: bool,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
}

pub struct WriterProxy {
    pub remote_writer_guid: GUID,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub data_max_size_serialized: i32,
}

