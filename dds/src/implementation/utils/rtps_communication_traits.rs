use rtps_pim::messages::submessages::{AckNackSubmessage, DataSubmessage, HeartbeatSubmessage};

use dds_transport::TransportWrite;

use crate::implementation::rtps::types::GuidPrefix;

pub trait ReceiveRtpsDataSubmessage {
    fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<'_>,
        source_guid_prefix: GuidPrefix,
    );
}

pub trait ReceiveRtpsHeartbeatSubmessage {
    fn on_heartbeat_submessage_received(
        &self,
        heartbeat_submessage: &HeartbeatSubmessage,
        source_guid_prefix: GuidPrefix,
    );
}

pub trait ReceiveRtpsAckNackSubmessage {
    fn on_acknack_submessage_received(
        &self,
        acknack_submessage: &AckNackSubmessage,
        source_guid_prefix: GuidPrefix,
    );
}

pub trait SendRtpsMessage {
    fn send_message(&self, transport: &mut impl TransportWrite);
}
