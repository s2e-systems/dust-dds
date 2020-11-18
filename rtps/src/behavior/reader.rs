use crate::structure::RtpsEndpoint;

pub struct RtpsReader {
    pub endpoint: RtpsEndpoint,
    pub expects_inline_qos: bool,
}

impl RtpsReader {
    pub fn new(endpoint: RtpsEndpoint, expects_inline_qos: bool) -> Self {
        Self {
            endpoint,
            expects_inline_qos,
        }
    }
}