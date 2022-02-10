use crate::{behavior::types::Duration, structure::endpoint::RtpsEndpointAttributes};

pub trait RtpsReaderAttributes: RtpsEndpointAttributes {
    type ReaderHistoryCacheType;

    fn heartbeat_response_delay(&self) -> &Duration;
    fn heartbeat_supression_duration(&self) -> &Duration;
    fn reader_cache(&self) -> &Self::ReaderHistoryCacheType;
    fn expects_inline_qos(&self) -> &bool;
}
