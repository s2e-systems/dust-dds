use rust_dds_api::infrastructure::qos::SubscriberQos;
use rust_rtps_pim::structure::types::GUIDPIM;

pub struct RTPSReaderGroupImpl<PSM>
where
    PSM: GUIDPIM,
{
    guid: PSM::GUIDType,
    qos: SubscriberQos,
}
