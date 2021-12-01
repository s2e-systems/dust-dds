use crate::{behavior::types::Duration, messages::{submessages::{
        AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage,
    }, types::Count}, structure::{
        history_cache::RtpsHistoryCacheConstructor,
        types::{Guid, ReliabilityKind, TopicKind},
    }};

use super::{reader_proxy::RtpsReaderProxy, writer::RtpsWriter};

pub struct RtpsStatefulWriter<L, C, R> {
    pub writer: RtpsWriter<L, C>,
    pub matched_readers: R,
}

impl<L, C, R> RtpsStatefulWriter<L, C, R>
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
            matched_readers: R::default(),
        }
    }
}

pub trait RtpsStatefulWriterOperations<L> {
    fn matched_reader_add(&mut self, a_reader_proxy: RtpsReaderProxy<L>);

    fn matched_reader_remove(&mut self, reader_proxy_guid: &Guid);

    fn matched_reader_lookup(&self, a_reader_guid: &Guid) -> Option<&RtpsReaderProxy<L>>;

    fn is_acked_by_all(&self) -> bool;
}

pub trait StatefulWriterBehaviorPerProxy<'a, S, P, D, L, C> {
    fn send_unsent_data(
        &mut self,
        writer: &'a RtpsWriter<L, C>,
        send_data: &mut dyn FnMut(DataSubmessage<P, D>),
        send_gap: &mut dyn FnMut(GapSubmessage<S>),
    );

    fn send_requested_data(
        &mut self,
        writer: &'a RtpsWriter<L, C>,
        send_data: &mut dyn FnMut(DataSubmessage<P, D>),
        send_gap: &mut dyn FnMut(GapSubmessage<S>),
    );

    fn send_heartbeat(
        &self,
        writer: &RtpsWriter<L, C>,
        heartbeat_count: Count,
        send_heartbeat: &mut dyn FnMut(HeartbeatSubmessage),
    );

    fn process_acknack_submessage(
        &mut self,
        writer: &'a RtpsWriter<L, C>,
        acknack: &AckNackSubmessage<S>,
    );
}
