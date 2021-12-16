use crate::{
    behavior::types::Duration,
    messages::submessages::{DataSubmessage, GapSubmessage},
    structure::{
        history_cache::RtpsHistoryCacheConstructor,
        types::{Guid, Locator, ReliabilityKind, TopicKind},
    },
};

use super::{reader_locator::RtpsReaderLocator, writer::RtpsWriter};

pub trait StatelessWriterAttributes {
    fn reader_locators(&self);
}

pub trait StatelessWriterConstructor<L> {
    fn new(
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
    ) -> Self;
}

pub struct RtpsStatelessWriter<L, C, R> {
    pub writer: RtpsWriter<L, C>,
    pub reader_locators: R,
}

impl<L, C, R> RtpsStatelessWriter<L, C, R>
where
    R: Default,
    C: RtpsHistoryCacheConstructor,
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
    fn reader_locator_add(&mut self, a_locator: RtpsReaderLocator);

    fn reader_locator_remove(&mut self, a_locator: &Locator);

    fn unsent_changes_reset(&mut self);
}

pub trait StatelessWriterBehavior<'a, S, P, D, L, C> {
    fn send_unsent_data(
        &mut self,
        writer: &'a RtpsWriter<L, C>,
        send_data: &mut dyn FnMut(DataSubmessage<P, D>),
        send_gap: &mut dyn FnMut(GapSubmessage<S>),
    );
}
