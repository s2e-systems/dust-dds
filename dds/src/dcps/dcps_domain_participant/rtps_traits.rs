use crate::{
    rtps::{stateful_writer::RtpsStatefulWriter, stateless_writer::RtpsStatelessWriter},
    transport::types::Guid,
};

pub trait RtpsWriter {
    fn guid(&self) -> Guid;
}

impl RtpsWriter for RtpsStatefulWriter {
    fn guid(&self) -> Guid {
        self.guid()
    }
}

impl RtpsWriter for RtpsStatelessWriter {
    fn guid(&self) -> Guid {
        self.guid()
    }
}
