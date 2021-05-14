use rust_dds_api::infrastructure::qos::SubscriberQos;
use rust_rtps_pim::structure::types::GUID;

pub struct RTPSReaderGroupImpl<'a, PSM: rust_rtps_pim::PIM> {
    guid: GUID<PSM>,
    qos: SubscriberQos<'a>,
}
