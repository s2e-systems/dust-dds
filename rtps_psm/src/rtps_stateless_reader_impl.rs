use std::ops::{Deref, DerefMut};

use rust_rtps_pim::{
    behavior::{reader::stateless_reader::RtpsStatelessReader, types::Duration},
    structure::{
        types::{Guid, Locator, ReliabilityKind, TopicKind},
        RtpsHistoryCacheOperations,
    },
};

pub struct RtpsStatelessReaderImpl<C>(RtpsStatelessReader<Vec<Locator>, C>);

impl<C> RtpsStatelessReaderImpl<C> {
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self
    where
        C: for<'a> RtpsHistoryCacheOperations<'a>,
    {
        Self(RtpsStatelessReader::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_response_delay,
            heartbeat_supression_duration,
            expects_inline_qos,
        ))
    }
}

impl<C> Deref for RtpsStatelessReaderImpl<C> {
    type Target = RtpsStatelessReader<Vec<Locator>, C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for RtpsStatelessReaderImpl<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
