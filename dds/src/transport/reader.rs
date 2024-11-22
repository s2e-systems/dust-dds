use super::cache_change::CacheChange;

pub trait ReaderHistoryCache: Send + Sync {
    fn add_change(&mut self, cache_change: CacheChange);
}

pub trait TransportReader: Send + Sync {
    fn guid(&self) -> [u8; 16];
    fn is_historical_data_received(&self) -> bool;
}
