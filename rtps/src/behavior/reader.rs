use crate::structure::RtpsEndpoint;
use crate::behavior::types::Duration;

pub struct RtpsReader {
    pub endpoint: RtpsEndpoint,
    pub expects_inline_qos: bool,
    pub heartbeat_response_delay: Duration,
}

impl RtpsReader {
    pub fn new(endpoint: RtpsEndpoint, expects_inline_qos: bool, heartbeat_response_delay: Duration) -> Self {
        Self {
            endpoint,
            expects_inline_qos,
            heartbeat_response_delay,
        }
    }
}