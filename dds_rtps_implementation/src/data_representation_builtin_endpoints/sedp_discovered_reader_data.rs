use std::io::Write;

use rust_dds_api::builtin_topics::SubscriptionBuiltinTopicData;
use rust_rtps_pim::{behavior::writer::reader_proxy::RtpsReaderProxy, structure::types::Locator};

use crate::dds_type::{DdsDeserialize, DdsSerialize, DdsType, Endianness};

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

impl DdsSerialize for SedpDiscoveredReaderData {
    fn serialize<W: Write, E: Endianness>(
        &self,
        _writer: W,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        todo!()
    }
}

impl DdsDeserialize<'_> for SedpDiscoveredReaderData {
    fn deserialize(_buf: &mut &'_ [u8]) -> rust_dds_api::return_type::DDSResult<Self> {
        todo!()
    }
}
