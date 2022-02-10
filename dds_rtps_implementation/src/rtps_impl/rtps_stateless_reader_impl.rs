use rust_rtps_pim::{
    behavior::{
        reader::{
            stateless_reader::{RtpsStatelessReaderAttributes, RtpsStatelessReaderConstructor},
        },
        stateless_reader_behavior::BestEffortStatelessReaderBehavior,
        types::Duration,
    },
    structure::{
        types::{Guid, Locator, ReliabilityKind, TopicKind},
    },
};

use super::{rtps_reader_history_cache_impl::ReaderHistoryCache, rtps_endpoint_impl::RtpsEndpointImpl, rtps_reader_impl::RtpsReaderImpl};

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
        heartbeat_supression_duration: Duration,
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
            heartbeat_supression_duration,
            expects_inline_qos,
        )
    }
}

impl<'a> IntoIterator for &'a mut RtpsStatelessReaderImpl {
    type Item = BestEffortStatelessReaderBehavior<'a, ReaderHistoryCache>;
    type IntoIter =
        std::option::IntoIter<BestEffortStatelessReaderBehavior<'a, ReaderHistoryCache>>;

    fn into_iter(self) -> Self::IntoIter {
        Some(self.into())
        .into_iter()
    }
}
