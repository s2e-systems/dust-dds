use crate::{
    behavior::types::Duration,
    structure::{
        history_cache::RtpsHistoryCacheConstructor,
        types::{Guid, ReliabilityKind, TopicKind, Locator},
    },
};

use super::{reader::RtpsReader, writer_proxy::RtpsWriterProxy};

pub struct RtpsStatefulReader<L, C, W> {
    pub reader: RtpsReader<L, C>,
    pub matched_writers: W,
}

impl<L, C, W> RtpsStatefulReader<L, C, W>
where
    C: RtpsHistoryCacheConstructor,
    W: Default,
{
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: L,
        multicast_locator_list: L,
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
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
            matched_writers: W::default(),
        }
    }
}

pub trait RtpsStatefulReaderAttributes {
    fn matched_writers(&self);
}

pub trait RtpsStatefulReaderConstructor {
    fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        heartbeat_response_delay: Duration,
        heartbeat_supression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self;
}

pub trait RtpsStatefulReaderOperations<L> {
    type WriterProxyType;

    fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy<L>);
    fn matched_writer_remove(&mut self, writer_proxy_guid: &Guid);
    fn matched_writer_lookup(&self, a_writer_guid: &Guid) -> Option<&Self::WriterProxyType>;
}
