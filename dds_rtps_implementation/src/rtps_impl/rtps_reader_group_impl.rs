use rust_dds_api::infrastructure::qos::SubscriberQos;
use rust_rtps_pim::structure::types::{EntityIdPIM, GuidPrefixPIM, GUIDPIM};

pub struct RTPSReaderGroupImpl<PSM: EntityIdPIM + GuidPrefixPIM + GUIDPIM<PSM> + Sized> {
    guid: PSM::GUIDType,
    qos: SubscriberQos,
}
