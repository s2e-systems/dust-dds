use rust_rtps::{behavior::Writer, structure::{Endpoint, Entity}};

pub struct WriterImpl;

impl Entity for WriterImpl {
    fn guid(&self) -> rust_rtps::types::GUID {
        todo!()
    }
}

impl Endpoint for WriterImpl {
    fn unicast_locator_list(&self) -> &[rust_rtps::types::Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[rust_rtps::types::Locator] {
        todo!()
    }

    fn topic_kind(&self) -> rust_rtps::types::TopicKind {
        todo!()
    }

    fn reliability_level(&self) -> rust_rtps::types::ReliabilityKind {
        todo!()
    }
}

impl Writer for WriterImpl {
    fn push_mode(&self) -> bool {
        todo!()
    }

    fn heartbeat_period(&self) -> rust_rtps::behavior::types::Duration {
        todo!()
    }

    fn nack_response_delay(&self) -> rust_rtps::behavior::types::Duration {
        todo!()
    }

    fn nack_suppression_duration(&self) -> rust_rtps::behavior::types::Duration {
        todo!()
    }

    fn last_change_sequence_number(&self) -> rust_rtps::types::SequenceNumber {
        todo!()
    }

    fn writer_cache(&mut self) -> &mut rust_rtps::structure::HistoryCache {
        todo!()
    }

    fn data_max_sized_serialized(&self) -> i32 {
        todo!()
    }

    fn new_change(
        &mut self,
        _kind: rust_rtps::types::ChangeKind,
        _data: rust_rtps::messages::submessages::submessage_elements::SerializedData,
        _inline_qos: rust_rtps::messages::submessages::submessage_elements::ParameterList,
        _handle: rust_rtps::types::InstanceHandle,
    ) -> rust_rtps::structure::CacheChange {
        todo!()
    }
}