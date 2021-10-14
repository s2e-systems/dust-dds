use crate::dds_type::DdsType;

pub struct SedpDiscoveredReaderData {}

impl DdsType for SedpDiscoveredReaderData {
    fn type_name() -> &'static str {
        "SedpDiscoveredReaderData"
    }

    fn has_key() -> bool {
        true
    }
}
