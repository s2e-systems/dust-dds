use rtps_pim::{
    messages::{
        submessage_elements::Parameter,
        submessages::{AckNackSubmessage, DataSubmessage, HeartbeatSubmessage},
    },
    structure::types::{GuidPrefix, SequenceNumber},
};

pub trait ReceiveRtpsDataSubmessage {
    fn on_data_submessage_received(
        &self,
        data_submessage: &DataSubmessage<Vec<Parameter>, &[u8]>,
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
        acknack_submessage: &AckNackSubmessage<Vec<SequenceNumber>>,
        source_guid_prefix: GuidPrefix,
    );
}
