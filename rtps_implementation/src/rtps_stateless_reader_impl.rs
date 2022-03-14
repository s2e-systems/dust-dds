use rtps_pim::{
    behavior::{
        reader::stateless_reader::{RtpsStatelessReaderAttributes, RtpsStatelessReaderConstructor},
        types::Duration,
    },
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

use super::{rtps_endpoint_impl::RtpsEndpointImpl, rtps_reader_impl::RtpsReaderImpl};

pub type RtpsStatelessReaderImpl = RtpsReaderImpl;

impl RtpsStatelessReaderAttributes for RtpsStatelessReaderImpl {}

impl RtpsStatelessReaderConstructor for RtpsStatelessReaderImpl {
    fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
        RtpsReaderImpl::new(
            RtpsEndpointImpl::new(
                guid,
                topic_kind,
                reliability_level,
                unicast_locator_list,
                multicast_locator_list,
            ),
            heartbeat_response_delay,
            heartbeat_suppression_duration,
            expects_inline_qos,
        )
    }
}
