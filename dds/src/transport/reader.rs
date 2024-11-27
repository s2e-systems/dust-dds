pub trait TransportReader: Send + Sync {
    fn guid(&self) -> [u8; 16];
    fn is_historical_data_received(&self) -> bool;
}
