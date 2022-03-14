use rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderAttributes,
            stateful_reader::{
                RtpsStatefulReaderAttributes, RtpsStatefulReaderConstructor,
                RtpsStatefulReaderOperations,
            },
            writer_proxy::RtpsWriterProxyAttributes,
        },
        types::Duration,
    },
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        types::{Guid, Locator, ReliabilityKind, TopicKind},
    },
};

use super::{
    rtps_endpoint_impl::RtpsEndpointImpl, rtps_history_cache_impl::RtpsHistoryCacheImpl,
    rtps_reader_impl::RtpsReaderImpl, rtps_writer_proxy_impl::RtpsWriterProxyImpl,
};

pub struct RtpsStatefulReaderImpl {
    pub reader: RtpsReaderImpl,
    pub matched_writers: Vec<RtpsWriterProxyImpl>,
}

impl RtpsEntityAttributes for RtpsStatefulReaderImpl {
    fn guid(&self) -> Guid {
        self.reader.endpoint.entity.guid
    }
}

impl RtpsEndpointAttributes for RtpsStatefulReaderImpl {
    fn topic_kind(&self) -> TopicKind {
        self.reader.endpoint.topic_kind
    }

    fn reliability_level(&self) -> ReliabilityKind {
        self.reader.endpoint.reliability_level
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.reader.endpoint.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.reader.endpoint.multicast_locator_list
    }
}

impl RtpsReaderAttributes for RtpsStatefulReaderImpl {
    type HistoryCacheType = RtpsHistoryCacheImpl;

    fn heartbeat_response_delay(&self) -> Duration {
        self.reader.heartbeat_response_delay
    }

    fn heartbeat_suppression_duration(&self) -> Duration {
        self.reader.heartbeat_suppression_duration
    }

    fn reader_cache(&mut self) -> &mut Self::HistoryCacheType {
        &mut self.reader.reader_cache
    }

    fn expects_inline_qos(&self) -> bool {
        self.reader.expects_inline_qos
    }
}

impl RtpsStatefulReaderAttributes for RtpsStatefulReaderImpl {
    type WriterProxyType = RtpsWriterProxyImpl;

    fn matched_writers(&self) -> &[Self::WriterProxyType] {
        &self.matched_writers
    }
}

impl RtpsStatefulReaderConstructor for RtpsStatefulReaderImpl {
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
        Self {
            reader: RtpsReaderImpl::new(
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
            ),
            matched_writers: Vec::new(),
        }
    }
}

impl RtpsStatefulReaderOperations for RtpsStatefulReaderImpl {
    type WriterProxyType = RtpsWriterProxyImpl;

    fn matched_writer_add(&mut self, a_writer_proxy: Self::WriterProxyType) {
        self.matched_writers.push(a_writer_proxy);
    }

    fn matched_writer_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&Self::WriterProxyType) -> bool,
    {
        self.matched_writers.retain(|x| !f(x))
    }

    fn matched_writer_lookup(&mut self, a_writer_guid: Guid) -> Option<&mut Self::WriterProxyType> {
        self.matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == a_writer_guid)
    }
}
