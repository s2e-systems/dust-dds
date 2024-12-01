use super::{history_cache::HistoryCache, types::Guid};

pub trait TransportWriter: Send + Sync {
    fn guid(&self) -> Guid;

    fn history_cache(&mut self) -> &mut dyn HistoryCache;

    fn is_change_acknowledged(&self, sequence_number: i64) -> bool;
}
