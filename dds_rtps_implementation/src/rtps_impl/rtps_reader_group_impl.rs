use rust_dds_api::infrastructure::qos::SubscriberQos;
use rust_rtps_pim::structure::types::{EntityIdType, GUIDType, GuidPrefixType};

pub struct RTPSReaderGroupImpl<PSM: EntityIdType + GuidPrefixType + GUIDType<PSM> + Sized> {
    guid: PSM::GUID,
    qos: SubscriberQos,
}
