use std::sync::{Arc, Mutex};

use crate::{
    implementation::data_representation_builtin_endpoints::spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    rtps::types::ChangeKind, topic_definition::type_support::DdsDeserialize,
};

use super::{
    discovery_types::{
        ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR, ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
        ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
    },
    participant::RtpsParticipant,
    reader::{ReaderCacheChange, ReaderHistoryCache, RtpsStatelessReader, TransportReader},
    stateful_writer::TransportWriter,
    stateless_writer::RtpsStatelessWriter,
    types::{Guid, TopicKind},
};

pub trait Transport: Send + Sync {
    fn create_participant_discovery_reader(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn TransportReader>>;

    fn create_participant_discovery_writer(&mut self) -> Arc<Mutex<dyn TransportWriter>>;

    fn create_topics_discovery_reader(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn TransportReader>>;

    fn create_topics_discovery_writer(&mut self) -> Arc<Mutex<dyn TransportWriter>>;

    fn create_publications_discovery_reader(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn TransportReader>>;

    fn create_publications_discovery_writer(&mut self) -> Arc<Mutex<dyn TransportWriter>>;

    fn create_subscriptions_discovery_reader(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn TransportReader>>;

    fn create_subscriptions_discovery_writer(&mut self) -> Arc<Mutex<dyn TransportWriter>>;

    fn create_user_defined_reader(
        &mut self,
        topic_kind: TopicKind,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn TransportReader>>;

    fn create_user_defined_writer(
        &mut self,
        topic_kind: TopicKind,
    ) -> Arc<Mutex<dyn TransportWriter>>;
}

pub struct RtpsTransport {
    rtps_participant: Arc<Mutex<RtpsParticipant>>,
}

impl RtpsTransport {
    pub fn new(rtps_participant: Arc<Mutex<RtpsParticipant>>) -> Self {
        Self { rtps_participant }
    }
}

pub struct RtpsSepdDiscovery {}

pub struct RtpsSpdpDiscovery {}

impl RtpsSpdpDiscovery {
    pub fn new() -> Self {
        // let spdp_builtin_participant_writer = RtpsStatelessWriter::new(
        //     Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER),
        //     message_sender,
        //     self.participant_proxy(),
        // );

        // let spdp_discovery_locator_list = [Locator::new(
        //     LOCATOR_KIND_UDP_V4,
        //     port_builtin_multicast(self.domain_id) as u32,
        //     DEFAULT_MULTICAST_LOCATOR_ADDRESS,
        // )];

        // for reader_locator in spdp_discovery_locator_list {
        //     stateless_writer.reader_locator_add(reader_locator);
        // }
        // let spdp_builtin_participant_reader = RtpsStatelessReader::new(
        //     Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER),
        //     spdp_builtin_participant_reader_history_cache,
        // );
        // Self {
        //     spdp_builtin_participant_writer,
        //     spdp_builtin_participant_reader,
        // }
        todo!()
    }
}

impl Transport for RtpsTransport {
    fn create_participant_discovery_reader(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn super::reader::TransportReader>> {
        struct SpdpDiscoveryReaderHistoryCache {
            rtps_participant: Arc<Mutex<RtpsParticipant>>,
            reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
        }
        impl ReaderHistoryCache for SpdpDiscoveryReaderHistoryCache {
            fn add_change(&mut self, cache_change: ReaderCacheChange) {
                match cache_change.kind {
                    ChangeKind::Alive => {
                        if let Ok(spdp_discovered_participant_data) =
                            SpdpDiscoveredParticipantData::deserialize_data(
                                cache_change.data_value.as_ref(),
                            )
                        {
                            self.rtps_participant
                                .lock()
                                .unwrap()
                                .add_discovered_participant(&spdp_discovered_participant_data);
                        }
                    }
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveDisposedUnregistered => {
                        todo!()
                    }
                    _ => (),
                }
                self.reader_history_cache.add_change(cache_change);
            }
        }
        let history_cache = Box::new(SpdpDiscoveryReaderHistoryCache {
            rtps_participant: self.rtps_participant.clone(),
            reader_history_cache,
        });
        let reader_guid = Guid::new(
            self.rtps_participant.lock().unwrap().guid().prefix(),
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
        );
        self.rtps_participant
            .lock()
            .unwrap()
            .create_builtin_stateless_reader(reader_guid, history_cache)
    }

    fn create_participant_discovery_writer(&mut self) -> Arc<Mutex<dyn TransportWriter>> {
        let writer_guid = Guid::new(
            self.rtps_participant.lock().unwrap().guid().prefix(),
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        );
        self.rtps_participant
            .lock()
            .unwrap()
            .create_builtin_stateless_writer(writer_guid)
    }

    fn create_topics_discovery_reader(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn super::reader::TransportReader>> {
        struct SedpTopicsDiscoveryReaderHistoryCache {
            reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
        }
        impl ReaderHistoryCache for SedpTopicsDiscoveryReaderHistoryCache {
            fn add_change(&mut self, cache_change: ReaderCacheChange) {
                self.reader_history_cache.add_change(cache_change);
            }
        }
        let reader_guid = Guid::new(
            self.rtps_participant.lock().unwrap().guid().prefix(),
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        );
        let history_cache = Box::new(SedpTopicsDiscoveryReaderHistoryCache {
            reader_history_cache,
        });
        self.rtps_participant
            .lock()
            .unwrap()
            .create_builtin_stateless_reader(reader_guid, history_cache)
    }

    fn create_topics_discovery_writer(&mut self) -> Arc<Mutex<dyn TransportWriter>> {
        let writer_guid = Guid::new(
            self.rtps_participant.lock().unwrap().guid().prefix(),
            ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        );
        self.rtps_participant
            .lock()
            .unwrap()
            .create_builtin_stateful_writer(writer_guid)
    }

    fn create_publications_discovery_reader(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn TransportReader>> {
        struct SedpPublicationsDiscoveryReaderHistoryCache {
            reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
        }
        impl ReaderHistoryCache for SedpPublicationsDiscoveryReaderHistoryCache {
            fn add_change(&mut self, cache_change: ReaderCacheChange) {
                self.reader_history_cache.add_change(cache_change);
            }
        }
        let reader_guid = Guid::new(
            self.rtps_participant.lock().unwrap().guid().prefix(),
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
        );
        let history_cache = Box::new(SedpPublicationsDiscoveryReaderHistoryCache {
            reader_history_cache,
        });
        self.rtps_participant
            .lock()
            .unwrap()
            .create_builtin_stateless_reader(reader_guid, history_cache)
    }

    fn create_publications_discovery_writer(&mut self) -> Arc<Mutex<dyn TransportWriter>> {
        let writer_guid = Guid::new(
            self.rtps_participant.lock().unwrap().guid().prefix(),
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        );
        self.rtps_participant
            .lock()
            .unwrap()
            .create_builtin_stateful_writer(writer_guid)
    }

    fn create_subscriptions_discovery_reader(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn TransportReader>> {
        struct SedpSubscriptionsDiscoveryReaderHistoryCache {
            reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
        }
        impl ReaderHistoryCache for SedpSubscriptionsDiscoveryReaderHistoryCache {
            fn add_change(&mut self, cache_change: ReaderCacheChange) {
                self.reader_history_cache.add_change(cache_change);
            }
        }
        let reader_guid = Guid::new(
            self.rtps_participant.lock().unwrap().guid().prefix(),
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        let history_cache = Box::new(SedpSubscriptionsDiscoveryReaderHistoryCache {
            reader_history_cache,
        });
        self.rtps_participant
            .lock()
            .unwrap()
            .create_builtin_stateless_reader(reader_guid, history_cache)
    }

    fn create_subscriptions_discovery_writer(&mut self) -> Arc<Mutex<dyn TransportWriter>> {
        let writer_guid = Guid::new(
            self.rtps_participant.lock().unwrap().guid().prefix(),
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        );
        self.rtps_participant
            .lock()
            .unwrap()
            .create_builtin_stateful_writer(writer_guid)
    }

    fn create_user_defined_reader(
        &mut self,
        topic_kind: TopicKind,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) -> Arc<Mutex<dyn super::reader::TransportReader>> {
        self.rtps_participant
            .lock()
            .unwrap()
            .create_reader(topic_kind, reader_history_cache)
    }

    fn create_user_defined_writer(
        &mut self,
        topic_kind: TopicKind,
    ) -> Arc<Mutex<dyn TransportWriter>> {
        self.rtps_participant
            .lock()
            .unwrap()
            .create_writer(topic_kind)
    }
}
