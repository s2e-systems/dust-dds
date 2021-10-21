use rust_dds_api::builtin_topics::SubscriptionBuiltinTopicData;
use rust_rtps_pim::{behavior::writer::reader_proxy::RtpsReaderProxy, structure::types::Locator};

use crate::dds_type::DdsType;

pub struct SedpDiscoveredReaderData {
    pub reader_proxy: RtpsReaderProxy<Vec<Locator>>,
    pub subscriptions_builtin_topic_data: SubscriptionBuiltinTopicData,
}

impl DdsType for SedpDiscoveredReaderData {
    fn type_name() -> &'static str {
        "SedpDiscoveredReaderData"
    }

    fn has_key() -> bool {
        true
    }
}
