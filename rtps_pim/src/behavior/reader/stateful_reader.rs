use core::ops::{Deref, DerefMut};

use crate::{
    behavior::types::Duration,
    structure::{
        types::{Guid, ReliabilityKind, TopicKind},
        RtpsHistoryCache,
    },
};

use super::{reader::RtpsReader, writer_proxy::RtpsWriterProxy};

pub struct RtpsStatefulReader<L, C, W> {
    reader: RtpsReader<L, C>,
    pub matched_writers: W,
}

impl<L, C, W> RtpsStatefulReader<L, C, W>
where
    W: Default,
    C: for<'a> RtpsHistoryCache<'a>,
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

impl<L, C, P> Deref for RtpsStatefulReader<L, C, P> {
    type Target = RtpsReader<L, C>;

    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl<L, C, P> DerefMut for RtpsStatefulReader<L, C, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}

pub trait RtpsStatefulReaderOperations<L> {
    fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy<L>);
    fn matched_writer_remove(&mut self, writer_proxy_guid: &Guid);
    fn matched_writer_lookup(&self, a_writer_guid: &Guid) -> Option<&RtpsWriterProxy<L>>;
}
