use super::history_cache::HistoryCache;

pub trait TransportWriter: Send + Sync {
    fn guid(&self) -> [u8; 16];

    fn history_cache(&mut self) -> &mut dyn HistoryCache;

    fn is_change_acknowledged(&self, sequence_number: i64) -> bool;
}
