use crate::{
    rtps::{
        stateful_reader::RtpsStatefulReader, stateful_writer::RtpsStatefulWriter,
        stateless_reader::RtpsStatelessReader, stateless_writer::RtpsStatelessWriter,
    },
    runtime::DdsRuntime,
    transport::{
        interface::WriteMessage,
        types::{CacheChange, Guid},
    },
};
use alloc::vec::Vec;

pub trait RtpsWriter {
    fn guid(&self) -> Guid;
    fn add_change(
        &mut self,
        cache_change: CacheChange,
        message_writer: &(impl WriteMessage + ?Sized),
        runtime: &impl DdsRuntime,
    );
}

impl RtpsWriter for RtpsStatefulWriter {
    fn guid(&self) -> Guid {
        self.guid()
    }

    fn add_change(
        &mut self,
        cache_change: CacheChange,
        message_writer: &(impl WriteMessage + ?Sized),
        runtime: &impl DdsRuntime,
    ) {
        self.add_change(cache_change, message_writer, &runtime.clock())
    }
}

impl RtpsWriter for RtpsStatelessWriter {
    fn guid(&self) -> Guid {
        self.guid()
    }

    fn add_change(
        &mut self,
        cache_change: CacheChange,
        message_writer: &(impl WriteMessage + ?Sized),
        _runtime: &impl DdsRuntime,
    ) {
        self.add_change(cache_change, message_writer)
    }
}

pub trait RtpsReader {
    fn changes_mut(&mut self) -> &mut Vec<CacheChange>;
}

impl RtpsReader for RtpsStatefulReader {
    fn changes_mut(&mut self) -> &mut Vec<CacheChange> {
        self.changes_mut()
    }
}

impl RtpsReader for RtpsStatelessReader {
    fn changes_mut(&mut self) -> &mut Vec<CacheChange> {
        self.changes_mut()
    }
}
