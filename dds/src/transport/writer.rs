use super::{cache_change::CacheChange, types::SequenceNumber};

pub trait WriterHistoryCache: Send + Sync {
    fn guid(&self) -> [u8; 16];

    fn add_change(&mut self, cache_change: CacheChange);

    fn remove_change(&mut self, sequence_number: SequenceNumber);

    fn is_change_acknowledged(&self, sequence_number: SequenceNumber) -> bool;
}
