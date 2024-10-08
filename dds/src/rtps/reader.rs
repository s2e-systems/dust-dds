use super::{
    endpoint::RtpsEndpoint,
    messages::{
        self,
        submessage_elements::{Data, ParameterList},
        submessages::data::DataSubmessage,
    },
    types::{DurabilityKind, Guid, GuidPrefix, Locator, ReliabilityKind},
    writer_proxy::RtpsWriterProxy,
};
use crate::implementation::{
    actor::ActorAddress, actors::message_sender_actor::MessageSenderActor,
    data_representation_builtin_endpoints::discovered_writer_data::WriterProxy,
};

pub struct ReaderCacheChange {
    pub writer_guid: Guid,
    pub source_timestamp: Option<messages::types::Time>,
    pub data_value: Data,
    pub inline_qos: ParameterList,
}

pub trait ReaderHistoryCache {
    fn add_change(&mut self, cache_change: ReaderCacheChange);
}

pub trait TransportReader {
    fn set_history_cache(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    );

    fn add_matched_writer(
        &mut self,
        writer_proxy: WriterProxy,
        reliability_kind: ReliabilityKind,
        durability_kind: DurabilityKind,
    );

    fn delete_matched_writer(&mut self, writer_guid: Guid);
}

pub struct RtpsReader {
    endpoint: RtpsEndpoint,
}

impl RtpsReader {
    pub fn new(endpoint: RtpsEndpoint) -> Self {
        Self { endpoint }
    }

    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.endpoint.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.endpoint.multicast_locator_list()
    }
}

pub struct RtpsStatelessReader {
    rtps_reader: RtpsReader,
    history_cache: Option<Box<dyn ReaderHistoryCache + Send + Sync + 'static>>,
}

impl RtpsStatelessReader {
    pub fn new(rtps_reader: RtpsReader) -> Self {
        Self {
            rtps_reader,
            history_cache: None,
        }
    }

    pub fn guid(&self) -> Guid {
        self.rtps_reader.guid()
    }
}

impl TransportReader for RtpsStatelessReader {
    fn set_history_cache(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) {
        self.history_cache = Some(reader_history_cache)
    }

    fn add_matched_writer(
        &mut self,
        _writer_proxy: WriterProxy,
        _reliability_kind: ReliabilityKind,
        _durability_kind: DurabilityKind,
    ) {
        // Do nothing
    }

    fn delete_matched_writer(&mut self, _writer_guid: Guid) {
        // Do nothing
    }
}

pub struct RtpsStatefulReader {
    rtps_reader: RtpsReader,
    matched_writers: Vec<RtpsWriterProxy>,
    history_cache: Option<Box<dyn ReaderHistoryCache + Send + Sync + 'static>>,
}

impl TransportReader for RtpsStatefulReader {
    fn set_history_cache(
        &mut self,
        reader_history_cache: Box<dyn ReaderHistoryCache + Send + Sync + 'static>,
    ) {
        self.history_cache = Some(reader_history_cache)
    }

    fn add_matched_writer(
        &mut self,
        writer_proxy: WriterProxy,
        reliability_kind: ReliabilityKind,
        _durability_kind: DurabilityKind,
    ) {
        if !self
            .matched_writers
            .iter()
            .any(|x| x.remote_writer_guid() == writer_proxy.remote_writer_guid)
        {
            let rtps_writer_proxy = RtpsWriterProxy::new(
                writer_proxy.remote_writer_guid,
                &writer_proxy.unicast_locator_list,
                &writer_proxy.multicast_locator_list,
                Some(writer_proxy.data_max_size_serialized),
                writer_proxy.remote_group_entity_id,
                reliability_kind,
            );
            self.matched_writers.push(rtps_writer_proxy);
        }
    }

    fn delete_matched_writer(&mut self, writer_guid: Guid) {
        self.matched_writers
            .retain(|x| x.remote_writer_guid() != writer_guid)
    }
}

impl RtpsStatefulReader {
    pub fn new(rtps_reader: RtpsReader) -> Self {
        Self {
            rtps_reader,
            matched_writers: Vec::new(),
            history_cache: None,
        }
    }

    pub fn matched_writer_lookup(&mut self, a_writer_guid: Guid) -> Option<&mut RtpsWriterProxy> {
        self.matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == a_writer_guid)
    }

    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<messages::types::Time>,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, data_submessage.writer_id());
        let sequence_number = data_submessage.writer_sn();
        if let Some(writer_proxy) = self.matched_writer_lookup(writer_guid) {
            match writer_proxy.reliability {
                ReliabilityKind::BestEffort => {
                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                    if sequence_number >= expected_seq_num {
                        writer_proxy.received_change_set(sequence_number);
                        if sequence_number > expected_seq_num {
                            writer_proxy.lost_changes_update(sequence_number);
                        }
                        if let Some(hc) = &mut self.history_cache {
                            let cache_change = ReaderCacheChange {
                                writer_guid,
                                source_timestamp,
                                data_value: data_submessage.serialized_payload().clone(),
                                inline_qos: data_submessage.inline_qos().clone(),
                            };
                            hc.add_change(cache_change);
                        }
                    }
                }
                ReliabilityKind::Reliable => {
                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                    if sequence_number == expected_seq_num {
                        writer_proxy.received_change_set(sequence_number);
                        if let Some(hc) = &mut self.history_cache {
                            let cache_change = ReaderCacheChange {
                                writer_guid,
                                source_timestamp,
                                data_value: data_submessage.serialized_payload().clone(),
                                inline_qos: data_submessage.inline_qos().clone(),
                            };
                            hc.add_change(cache_change);
                        }
                    }
                }
            }
        }
    }
}

// The methods in this impl block are not defined by the standard
impl RtpsStatefulReader {
    pub fn is_historical_data_received(&self) -> bool {
        !self
            .matched_writers
            .iter()
            .any(|p| !p.is_historical_data_received())
    }

    pub fn send_message(&mut self, message_sender_actor: &ActorAddress<MessageSenderActor>) {
        for writer_proxy in self.matched_writers.iter_mut() {
            writer_proxy.send_message(&self.rtps_reader.guid(), message_sender_actor)
        }
    }
}
