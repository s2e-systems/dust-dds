use core::ops::{Deref, DerefMut};

use crate::{
    behavior::types::Duration,
    structure::{
        types::{Guid, ReliabilityKind, TopicKind},
        RtpsEndpoint, RtpsHistoryCache,
    },
};

pub struct RtpsReader<L, C> {
    endpoint: RtpsEndpoint<L>,
    pub heartbeat_response_delay: Duration,
    pub heartbeat_supression_duration: Duration,
    pub reader_cache: C,
    pub expects_inline_qos: bool,
}

impl<L, C> Deref for RtpsReader<L, C> {
    type Target = RtpsEndpoint<L>;

    fn deref(&self) -> &Self::Target {
        &self.endpoint
    }
}

impl<L, C> DerefMut for RtpsReader<L, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.endpoint
    }
}

impl<L, C> RtpsReader<L, C> {
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: L,
        multicast_locator_list: L,
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self
    where
        C: for<'a> RtpsHistoryCache<'a>,
    {
        Self {
            endpoint: RtpsEndpoint::new(
                guid,
                topic_kind,
                reliability_level,
                unicast_locator_list,
                multicast_locator_list,
            ),
            heartbeat_response_delay,
            heartbeat_supression_duration,
            reader_cache: C::new(),
            expects_inline_qos,
        }
    }
}
