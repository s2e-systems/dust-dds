use core::ops::{Deref, DerefMut};

use crate::{
    behavior::types::Duration,
    structure::{
        types::{Guid, ReliabilityKind, TopicKind},
        RtpsHistoryCache,
    },
};

use super::reader::RtpsReader;

pub struct RtpsStatelessReader<L, C> {
    reader: RtpsReader<L, C>,
}

impl<L, C> Deref for RtpsStatelessReader<L, C> {
    type Target = RtpsReader<L, C>;

    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl<L, C> DerefMut for RtpsStatelessReader<L, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}

impl<L, C> RtpsStatelessReader<L, C> {
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
            reader: RtpsReader::new(
                guid,
                topic_kind,
                reliability_level,
                unicast_locator_list,
                multicast_locator_list,
                heartbeat_response_delay,
                heartbeat_supression_duration,
                expects_inline_qos,
            ),
        }
    }
}
