use rust_dds_api::infrastructure::qos::SubscriberQos;
use rust_rtps_pim::structure::types::{EntityIdType, GuidPrefixType, GUID};

pub struct RTPSReaderGroupImpl<PSM: EntityIdType + GuidPrefixType> {
    guid: GUID<PSM>,
    qos: SubscriberQos,
}
