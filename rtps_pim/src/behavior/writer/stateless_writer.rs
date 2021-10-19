use core::ops::{Deref, DerefMut};

use crate::{
    behavior::types::Duration,
    structure::{
        types::{Guid, Locator, ReliabilityKind, TopicKind},
        RtpsHistoryCache,
    },
};

use super::writer::RtpsWriter;

pub struct RtpsStatelessWriter<L, C, R> {
    pub writer: RtpsWriter<L, C>, // Temporarily left as pub because of borrow checker issues with the behaviour
    pub reader_locators: R,
}

impl<L, C, R> Deref for RtpsStatelessWriter<L, C, R> {
    type Target = RtpsWriter<L, C>;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<L, C, R> DerefMut for RtpsStatelessWriter<L, C, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl<L, C, R> RtpsStatelessWriter<L, C, R>
where
    R: Default,
    C: for<'a> RtpsHistoryCache<'a>,
{
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: L,
        multicast_locator_list: L,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self {
        Self {
            writer: RtpsWriter::new(
                guid,
                topic_kind,
                reliability_level,
                unicast_locator_list,
                multicast_locator_list,
                push_mode,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
                data_max_size_serialized,
            ),
            reader_locators: R::default(),
        }
    }
}

pub trait RtpsStatelessWriterOperations {
    fn reader_locator_add(&mut self, a_locator: Locator);

    fn reader_locator_remove(&mut self, a_locator: &Locator);

    fn unsent_changes_reset(&mut self);
}
