use rust_dds_api::infrastructure::qos::SubscriberQos;
use rust_rtps_pim::structure::types::GUID;

pub struct RTPSReaderGroupImpl<PSM> {
    guid: GUID<PSM>,
    qos: SubscriberQos,
}
