use rust_dds_api::infrastructure::qos::SubscriberQos;
use rust_rtps_pim::structure::types::{EntityIdPIM, GUIDType, GuidPrefixPIM};

pub struct RTPSReaderGroupImpl<PSM: EntityIdPIM + GuidPrefixPIM + GUIDType<PSM> + Sized> {
    guid: PSM::GUID,
    qos: SubscriberQos,
}
