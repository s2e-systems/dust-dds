use rtps_pim::{
    messages::{
        overall_structure::RtpsSubmessageType,
        submessage_elements::Parameter,
        submessages::{AckNackSubmessage, DataSubmessage, HeartbeatSubmessage},
        types::FragmentNumber,
    },
    structure::types::{GuidPrefix, Locator, SequenceNumber},
    transport::TransportWrite,
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

pub trait SendRtpsMessage {
    fn send_message(
        &self,
        transport: &mut impl for<'a> TransportWrite<
            Vec<
                RtpsSubmessageType<
                    Vec<SequenceNumber>,
                    Vec<Parameter<'a>>,
                    &'a [u8],
                    Vec<Locator>,
                    Vec<FragmentNumber>,
                >,
            >,
        >,
    );
}
