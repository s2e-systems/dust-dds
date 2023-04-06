use crate::subscription::data_reader::AnyDataReader;

#[derive(PartialEq, Debug)]
pub struct ListenerDataReaderNode();

impl ListenerDataReaderNode {
    pub fn new() -> Self {
        Self()
    }
}

impl AnyDataReader for ListenerDataReaderNode {}
