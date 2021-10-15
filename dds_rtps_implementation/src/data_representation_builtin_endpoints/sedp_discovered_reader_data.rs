use crate::{dds_type::DdsType, rtps_impl::rtps_reader_proxy_impl::RtpsReaderProxyImpl};

pub struct SedpDiscoveredReaderData {
    pub reader_proxy: RtpsReaderProxyImpl,
    pub subscriptions_builtin_topic_data: (),
}

impl DdsType for SedpDiscoveredReaderData {
    fn type_name() -> &'static str {
        "SedpDiscoveredReaderData"
    }

    fn has_key() -> bool {
        true
    }
}
