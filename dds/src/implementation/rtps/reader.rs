use super::{
    endpoint::RtpsEndpoint,
    types::{Guid, Locator},
};
use crate::infrastructure::time::Duration;

pub struct RtpsReader {
    endpoint: RtpsEndpoint,
    _heartbeat_response_delay: Duration,
    _heartbeat_suppression_duration: Duration,
    _expects_inline_qos: bool,
}

impl RtpsReader {
    pub fn new(
        endpoint: RtpsEndpoint,
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            endpoint,
            _heartbeat_response_delay: heartbeat_response_delay,
            _heartbeat_suppression_duration: heartbeat_suppression_duration,
            _expects_inline_qos: expects_inline_qos,
        }
    }

    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.endpoint.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.endpoint.multicast_locator_list()
    }
}
