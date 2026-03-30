use crate::{rtps::history_cache::HistoryCache, transport::types::Guid};
use alloc::boxed::Box;

pub struct RtpsStatelessReader {
    guid: Guid,
    history_cache: Box<dyn HistoryCache + Send + Sync>,
}

impl RtpsStatelessReader {
    pub fn new(guid: Guid, history_cache: Box<dyn HistoryCache + Send + Sync>) -> Self {
        Self {
            guid,
            history_cache,
        }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn history_cache_mut(&mut self) -> &mut Box<dyn HistoryCache + Send + Sync> {
        &mut self.history_cache
    }
}
