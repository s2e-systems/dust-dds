use crate::publication::data_writer::AnyDataWriter;

#[derive(PartialEq, Debug, Eq, Default)]
pub struct ListenerDataWriterNode();

impl ListenerDataWriterNode {
    pub fn new() -> Self {
        Self()
    }
}

impl AnyDataWriter for ListenerDataWriterNode {}
