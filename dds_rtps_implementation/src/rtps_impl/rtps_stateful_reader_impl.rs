use rust_rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderAttributes,
            stateful_reader::{
                RtpsStatefulReaderConstructor,
                RtpsStatefulReaderOperations,
                RtpsStatefulReaderAttributes
            },
            writer_proxy::RtpsWriterProxyAttributes,
        },
        stateful_reader_behavior::{StatefulReaderBehavior, BestEffortStatefulReaderBehavior, ReliableStatefulReaderBehavior},
        types::Duration,
    },
    structure::{
        endpoint::RtpsEndpointAttributes,
        entity::RtpsEntityAttributes,
        types::{Guid, Locator, ReliabilityKind, TopicKind},
    },
};

use super::{
    rtps_reader_history_cache_impl::ReaderHistoryCache,
    rtps_writer_proxy_impl::RtpsWriterProxyImpl,
    rtps_endpoint_impl::RtpsEndpointImpl, rtps_reader_impl::RtpsReaderImpl,
};

pub struct RtpsStatefulReaderImpl {
    pub reader: RtpsReaderImpl,
    pub matched_writers: Vec<RtpsWriterProxyImpl>,
}

impl RtpsEntityAttributes for RtpsStatefulReaderImpl {
    fn guid(&self) -> &Guid {
        &self.reader.endpoint.entity.guid
    }
}

impl RtpsEndpointAttributes for RtpsStatefulReaderImpl {
    fn topic_kind(&self) -> &TopicKind {
        &self.reader.endpoint.topic_kind
    }

    fn reliability_level(&self) -> &ReliabilityKind {
        &self.reader.endpoint.reliability_level
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.reader.endpoint.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.reader.endpoint.multicast_locator_list
    }
}

impl RtpsReaderAttributes for RtpsStatefulReaderImpl {
    type ReaderHistoryCacheType = ReaderHistoryCache;

    fn heartbeat_response_delay(&self) -> &Duration {
        &self.reader.heartbeat_response_delay
    }

    fn heartbeat_supression_duration(&self) -> &Duration {
        &self.reader.heartbeat_supression_duration
    }

    fn reader_cache(&self) -> &Self::ReaderHistoryCacheType {
        &self.reader.reader_cache
    }

    fn expects_inline_qos(&self) -> &bool {
        &self.reader.expects_inline_qos
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
        heartbeat_supression_duration: Duration,
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
                heartbeat_supression_duration,
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

    fn matched_writer_remove(&mut self, writer_proxy_guid: &Guid) {
        self.matched_writers
            .retain(|x| x.remote_writer_guid() != writer_proxy_guid)
    }

    fn matched_writer_lookup(&self, a_writer_guid: &Guid) -> Option<&Self::WriterProxyType> {
        self.matched_writers
            .iter()
            .find(|&x| x.remote_writer_guid() == a_writer_guid)
    }
}

impl RtpsStatefulReaderImpl {
    pub fn behavior<'a>(&'a mut self) -> Option<StatefulReaderBehavior<'a, RtpsWriterProxyImpl, ReaderHistoryCache>> {
        match self.reliability_level() {
            ReliabilityKind::BestEffort => {
                Some(StatefulReaderBehavior::BestEffort(
                    BestEffortStatefulReaderBehavior
                ))
            },

            ReliabilityKind::Reliable => {
                Some(StatefulReaderBehavior::<'a, RtpsWriterProxyImpl, ReaderHistoryCache>::Reliable(
                    ReliableStatefulReaderBehavior::<'a, RtpsWriterProxyImpl, ReaderHistoryCache> {
                        writer_proxy: self.matched_writers.iter_mut().next()?,
                        reader_cache: &mut self.reader.reader_cache,
                    }
                ))
            },
        }
    }
}

// impl<'a> IntoIterator for &'a mut RtpsStatefulReaderImpl {
//     type Item = StatefulReaderBehavior<'a, RtpsWriterProxyImpl, ReaderHistoryCache>;
//     type IntoIter = StatefulReaderIterator<'a>;

//     fn into_iter(self) -> Self::IntoIter {
//         StatefulReaderIterator {
//             reliability_level: self.reader.endpoint.reliability_level.clone(),
//             writer_proxy: self.matched_writers.as_slice().iter_mut(),
//             reader_cache: &mut self.reader.reader_cache,
//         }
//     }
// }

// pub struct StatefulReaderIterator<'a> {
//     reliability_level: ReliabilityKind,
//     writer_proxy: core::slice::IterMut<'a, RtpsWriterProxyImpl>,
//     reader_cache: &'a mut ReaderHistoryCache,
// }

// impl<'a> Iterator for StatefulReaderIterator<'a> {
//     type Item = StatefulReaderBehavior<'a, RtpsWriterProxyImpl, ReaderHistoryCache>;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self.reliability_level {
//             ReliabilityKind::BestEffort => {
//                 Some(StatefulReaderBehavior::BestEffort(
//                     BestEffortStatefulReaderBehavior
//                 ))
//             }
//             ReliabilityKind::Reliable => {
//                 Some(StatefulReaderBehavior::<'a, RtpsWriterProxyImpl, ReaderHistoryCache>::Reliable(
//                     ReliableStatefulReaderBehavior::<'a, RtpsWriterProxyImpl, ReaderHistoryCache> {
//                         writer_proxy: self.writer_proxy.next()?,
//                         reader_cache: self.reader_cache,
//                     }
//                 ))
//             },
//         }
//     }
// }
