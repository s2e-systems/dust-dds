use crate::{
    rtps::{
        cache_change::CacheChangeWrite, stateful_writer::RtpsStatefulWriter,
        stateless_writer::RtpsStatelessWriter,
    },
    transport::types::Guid,
};

pub trait RtpsWriter {
    fn guid(&self) -> Guid;
    fn add_change(&mut self, cache_change: CacheChangeWrite);
}

impl RtpsWriter for RtpsStatefulWriter {
    fn guid(&self) -> Guid {
        self.guid()
    }

    fn add_change(&mut self, cache_change: CacheChangeWrite) {
        self.add_change(cache_change)
    }
}

impl RtpsWriter for RtpsStatelessWriter {
    fn guid(&self) -> Guid {
        self.guid()
    }

    fn add_change(&mut self, cache_change: CacheChangeWrite) {
        self.add_change(cache_change)
    }
}
