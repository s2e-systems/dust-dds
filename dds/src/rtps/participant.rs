use crate::{
    rtps::{message_receiver::MessageReceiver, stateful_writer::RtpsStatefulWriter},
    runtime::{
        actor::{ActorAddress, Mail, MailHandler},
        executor::block_on,
    },
    transport::{
        history_cache::{CacheChange, HistoryCache},
        reader::WriterProxy,
        types::{Guid, Locator, ProtocolVersion, SequenceNumber, VendorId},
        writer::{ReaderProxy, TransportStatefulWriter, TransportStatelessWriter},
    },
};

use super::{
    error::RtpsResult,
    message_sender::MessageSender,
    messages::overall_structure::RtpsMessageRead,
    stateful_reader::RtpsStatefulReader,
    stateless_reader::RtpsStatelessReader,
    stateless_writer::RtpsStatelessWriter,
    types::{PROTOCOLVERSION_2_4, VENDOR_ID_S2E},
};

pub struct RtpsParticipant {
    guid: Guid,
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    stateless_writer_list: Vec<RtpsStatelessWriter>,
    stateful_writer_list: Vec<RtpsStatefulWriter>,
    stateless_reader_list: Vec<RtpsStatelessReader>,
    stateful_reader_list: Vec<RtpsStatefulReader>,
    message_sender: MessageSender,
}

impl RtpsParticipant {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        guid: Guid,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        metatraffic_unicast_locator_list: Vec<Locator>,
        metatraffic_multicast_locator_list: Vec<Locator>,
    ) -> RtpsResult<Self> {
        let guid_prefix = guid.prefix();
        let message_sender =
            MessageSender::new(guid_prefix, std::net::UdpSocket::bind("0.0.0.0:0000")?);

        Ok(Self {
            guid,
            protocol_version: PROTOCOLVERSION_2_4,
            vendor_id: VENDOR_ID_S2E,
            default_unicast_locator_list,
            default_multicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            stateless_writer_list: vec![],
            stateful_writer_list: vec![],
            stateless_reader_list: vec![],
            stateful_reader_list: vec![],

            message_sender,
        })
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    pub fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    pub fn default_unicast_locator_list(&self) -> &[Locator] {
        self.default_unicast_locator_list.as_slice()
    }

    pub fn set_default_unicast_locator_list(&mut self, list: Vec<Locator>) {
        self.default_unicast_locator_list = list;
    }

    pub fn default_multicast_locator_list(&self) -> &[Locator] {
        self.default_multicast_locator_list.as_slice()
    }

    pub fn set_default_multicast_locator_list(&mut self, list: Vec<Locator>) {
        self.default_multicast_locator_list = list;
    }

    pub fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        self.metatraffic_unicast_locator_list.as_ref()
    }

    pub fn set_metatraffic_unicast_locator_list(&mut self, list: Vec<Locator>) {
        self.metatraffic_unicast_locator_list = list;
    }

    pub fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        self.metatraffic_multicast_locator_list.as_ref()
    }

    pub fn set_metatraffic_multicast_locator_list(&mut self, list: Vec<Locator>) {
        self.metatraffic_multicast_locator_list = list;
    }

    pub fn create_stateless_writer(&mut self, writer_guid: Guid) {
        let writer = RtpsStatelessWriter::new(writer_guid);
        self.stateless_writer_list.push(writer);
    }

    pub fn create_stateful_writer(&mut self, writer_guid: Guid, data_max_size_serialized: usize) {
        let writer = RtpsStatefulWriter::new(writer_guid, data_max_size_serialized);
        self.stateful_writer_list.push(writer);
    }

    pub fn delete_writer(&mut self, writer_guid: Guid) {
        self.stateful_writer_list
            .retain(|x| x.guid() != writer_guid);
    }

    pub fn create_stateful_reader(
        &mut self,
        reader_guid: Guid,
        reader_history_cache: Box<dyn HistoryCache>,
    ) {
        let reader = RtpsStatefulReader::new(reader_guid, reader_history_cache);

        self.stateful_reader_list.push(reader);
    }

    pub fn create_stateless_reader(
        &mut self,
        reader_guid: Guid,
        reader_history_cache: Box<dyn HistoryCache>,
    ) {
        let reader = RtpsStatelessReader::new(reader_guid, reader_history_cache);

        self.stateless_reader_list.push(reader);
    }

    pub fn delete_reader(&mut self, reader_guid: Guid) {
        self.stateful_reader_list
            .retain(|x| x.guid() != reader_guid);
    }

    pub fn process_builtin_rtps_message(&mut self, message: RtpsMessageRead) {
        MessageReceiver::new(message).process_message(
            &mut self.stateless_reader_list,
            &mut self.stateful_reader_list,
            &mut self.stateful_writer_list,
            &self.message_sender,
        );
    }

    pub fn process_user_defined_rtps_message(&mut self, message: RtpsMessageRead) {
        MessageReceiver::new(message).process_message(
            &mut self.stateless_reader_list,
            &mut self.stateful_reader_list,
            &mut self.stateful_writer_list,
            &self.message_sender,
        );
    }
}

pub struct ProcessBuiltinRtpsMessage {
    pub rtps_message: RtpsMessageRead,
}
impl Mail for ProcessBuiltinRtpsMessage {
    type Result = ();
}
impl MailHandler<ProcessBuiltinRtpsMessage> for RtpsParticipant {
    fn handle(
        &mut self,
        message: ProcessBuiltinRtpsMessage,
    ) -> <ProcessBuiltinRtpsMessage as Mail>::Result {
        self.process_builtin_rtps_message(message.rtps_message);
    }
}

pub struct ProcessUserDefinedRtpsMessage {
    pub rtps_message: RtpsMessageRead,
}
impl Mail for ProcessUserDefinedRtpsMessage {
    type Result = ();
}
impl MailHandler<ProcessUserDefinedRtpsMessage> for RtpsParticipant {
    fn handle(
        &mut self,
        message: ProcessUserDefinedRtpsMessage,
    ) -> <ProcessUserDefinedRtpsMessage as Mail>::Result {
        self.process_user_defined_rtps_message(message.rtps_message);
    }
}

pub struct SendHeartbeat;
impl Mail for SendHeartbeat {
    type Result = ();
}
impl MailHandler<SendHeartbeat> for RtpsParticipant {
    fn handle(&mut self, _: SendHeartbeat) -> <SendHeartbeat as Mail>::Result {
        for writer in self.stateful_writer_list.iter_mut() {
            writer.send_message(&self.message_sender);
        }
    }
}

pub struct CreateStatefulWriter {
    pub writer_guid: Guid,
    pub data_max_size_serialized: usize,
    pub rtps_participant_address: ActorAddress<RtpsParticipant>,
}

impl Mail for CreateStatefulWriter {
    type Result = Box<dyn TransportStatefulWriter>;
}
impl MailHandler<CreateStatefulWriter> for RtpsParticipant {
    fn handle(&mut self, message: CreateStatefulWriter) -> <CreateStatefulWriter as Mail>::Result {
        self.create_stateful_writer(message.writer_guid, message.data_max_size_serialized);

        struct RtpsUserDefinedWriterHistoryCache {
            rtps_participant_address: ActorAddress<RtpsParticipant>,
            guid: Guid,
        }
        impl TransportStatefulWriter for RtpsUserDefinedWriterHistoryCache {
            fn guid(&self) -> Guid {
                self.guid
            }

            fn history_cache(&mut self) -> &mut dyn HistoryCache {
                self
            }

            fn is_change_acknowledged(&self, sequence_number: SequenceNumber) -> bool {
                block_on(
                    self.rtps_participant_address
                        .send_actor_mail(IsChangeAcknowledged {
                            guid: self.guid,
                            sequence_number,
                        })
                        .expect("Actor must exist")
                        .receive_reply(),
                )
            }

            fn add_matched_reader(&mut self, reader_proxy: ReaderProxy) {
                block_on(
                    self.rtps_participant_address
                        .send_actor_mail(AddMatchedReader {
                            writer: self.guid,
                            reader_proxy,
                        })
                        .expect("Actor must exist")
                        .receive_reply(),
                )
            }

            fn remove_matched_reader(&mut self, remote_reader_guid: Guid) {
                todo!()
            }
        }
        impl HistoryCache for RtpsUserDefinedWriterHistoryCache {
            fn add_change(&mut self, cache_change: CacheChange) {
                self.rtps_participant_address
                    .send_actor_mail(AddStatefulWriterCacheChange {
                        writer: self.guid,
                        cache_change,
                    })
                    .ok();
            }

            fn remove_change(&mut self, sequence_number: SequenceNumber) {
                todo!()
                //     self.rtps_participant_address
                //         .send_actor_mail(RemoveCacheChange {
                //             guid: self.guid,
                //             sequence_number,
                //         })
                //         .ok();
            }
        }

        Box::new(RtpsUserDefinedWriterHistoryCache {
            rtps_participant_address: message.rtps_participant_address,
            guid: message.writer_guid,
        })
    }
}

pub struct CreateStatelessWriter {
    pub writer_guid: Guid,
    pub rtps_participant_address: ActorAddress<RtpsParticipant>,
}

impl Mail for CreateStatelessWriter {
    type Result = Box<dyn TransportStatelessWriter>;
}
impl MailHandler<CreateStatelessWriter> for RtpsParticipant {
    fn handle(
        &mut self,
        message: CreateStatelessWriter,
    ) -> <CreateStatelessWriter as Mail>::Result {
        self.create_stateless_writer(message.writer_guid);

        struct StatelessWriter {
            rtps_participant_address: ActorAddress<RtpsParticipant>,
            guid: Guid,
        }
        impl TransportStatelessWriter for StatelessWriter {
            fn guid(&self) -> Guid {
                self.guid
            }

            fn history_cache(&mut self) -> &mut dyn HistoryCache {
                self
            }

            fn add_reader_locator(&mut self, locator: Locator) {
                self.rtps_participant_address
                    .send_actor_mail(AddReaderLocator {
                        writer: self.guid,
                        locator,
                    })
                    .ok();
            }

            fn remove_reader_locator(&mut self, locator: &Locator) {
                todo!()
            }
        }
        impl HistoryCache for StatelessWriter {
            fn add_change(&mut self, cache_change: CacheChange) {
                self.rtps_participant_address
                    .send_actor_mail(AddStatelessWriterCacheChange {
                        writer: self.guid,
                        cache_change,
                    })
                    .ok();
            }

            fn remove_change(&mut self, sequence_number: SequenceNumber) {
                self.rtps_participant_address
                    .send_actor_mail(RemoveStatelessWriterCacheChange {
                        writer: self.guid,
                        sequence_number,
                    })
                    .ok();
            }
        }

        Box::new(StatelessWriter {
            rtps_participant_address: message.rtps_participant_address,
            guid: message.writer_guid,
        })
    }
}

pub struct CreateStatefulReader {
    pub reader_guid: Guid,
    pub reader_history_cache: Box<dyn HistoryCache>,
}

impl Mail for CreateStatefulReader {
    type Result = ();
}
impl MailHandler<CreateStatefulReader> for RtpsParticipant {
    fn handle(&mut self, message: CreateStatefulReader) -> <CreateStatefulReader as Mail>::Result {
        self.create_stateful_reader(message.reader_guid, message.reader_history_cache)
    }
}

pub struct CreateStatelessReader {
    pub reader_guid: Guid,
    pub reader_history_cache: Box<dyn HistoryCache>,
}

impl Mail for CreateStatelessReader {
    type Result = ();
}
impl MailHandler<CreateStatelessReader> for RtpsParticipant {
    fn handle(
        &mut self,
        message: CreateStatelessReader,
    ) -> <CreateStatelessReader as Mail>::Result {
        self.create_stateless_reader(message.reader_guid, message.reader_history_cache)
    }
}

pub struct AddReaderLocator {
    pub writer: Guid,
    pub locator: Locator,
}
impl Mail for AddReaderLocator {
    type Result = ();
}
impl MailHandler<AddReaderLocator> for RtpsParticipant {
    fn handle(&mut self, message: AddReaderLocator) -> <AddReaderLocator as Mail>::Result {
        if let Some(w) = self
            .stateless_writer_list
            .iter_mut()
            .find(|x| x.guid() == message.writer)
        {
            w.reader_locator_add(message.locator);
        }
    }
}

pub struct AddMatchedReader {
    pub writer: Guid,
    pub reader_proxy: ReaderProxy,
}
impl Mail for AddMatchedReader {
    type Result = ();
}
impl MailHandler<AddMatchedReader> for RtpsParticipant {
    fn handle(&mut self, message: AddMatchedReader) -> <AddMatchedReader as Mail>::Result {
        if let Some(w) = self
            .stateful_writer_list
            .iter_mut()
            .find(|x| x.guid() == message.writer)
        {
            let mut reader_proxy = message.reader_proxy;
            if reader_proxy.unicast_locator_list.is_empty() {
                reader_proxy.unicast_locator_list = self.default_unicast_locator_list.clone();
            }

            if reader_proxy.unicast_locator_list.is_empty() {
                reader_proxy.multicast_locator_list = self.default_multicast_locator_list.clone();
            }

            w.add_matched_reader(&reader_proxy);
        }
    }
}

pub struct AddMatchedWriter {
    pub reader: Guid,
    pub writer_proxy: WriterProxy,
}
impl Mail for AddMatchedWriter {
    type Result = ();
}
impl MailHandler<AddMatchedWriter> for RtpsParticipant {
    fn handle(&mut self, message: AddMatchedWriter) -> <AddMatchedWriter as Mail>::Result {
        if let Some(r) = self
            .stateful_reader_list
            .iter_mut()
            .find(|x| x.guid() == message.reader)
        {
            let mut writer_proxy = message.writer_proxy;
            if writer_proxy.unicast_locator_list.is_empty() {
                writer_proxy.unicast_locator_list = self.default_unicast_locator_list.clone();
            }

            if writer_proxy.unicast_locator_list.is_empty() {
                writer_proxy.multicast_locator_list = self.default_multicast_locator_list.clone();
            }

            r.add_matched_writer(&writer_proxy);
        }
    }
}

pub struct AddStatelessWriterCacheChange {
    pub writer: Guid,
    pub cache_change: CacheChange,
}
impl Mail for AddStatelessWriterCacheChange {
    type Result = ();
}
impl MailHandler<AddStatelessWriterCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: AddStatelessWriterCacheChange,
    ) -> <AddStatelessWriterCacheChange as Mail>::Result {
        if let Some(w) = self
            .stateless_writer_list
            .iter_mut()
            .find(|dw| dw.guid() == message.writer)
        {
            w.add_change(message.cache_change);
            w.send_message(&self.message_sender);
        }
    }
}

pub struct RemoveStatelessWriterCacheChange {
    pub writer: Guid,
    pub sequence_number: SequenceNumber,
}
impl Mail for RemoveStatelessWriterCacheChange {
    type Result = ();
}
impl MailHandler<RemoveStatelessWriterCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: RemoveStatelessWriterCacheChange,
    ) -> <RemoveStatelessWriterCacheChange as Mail>::Result {
        if let Some(w) = self
            .stateless_writer_list
            .iter_mut()
            .find(|dw| dw.guid() == message.writer)
        {
            w.remove_change(message.sequence_number);
        }
    }
}

pub struct AddStatefulWriterCacheChange {
    pub writer: Guid,
    pub cache_change: CacheChange,
}
impl Mail for AddStatefulWriterCacheChange {
    type Result = ();
}
impl MailHandler<AddStatefulWriterCacheChange> for RtpsParticipant {
    fn handle(
        &mut self,
        message: AddStatefulWriterCacheChange,
    ) -> <AddStatefulWriterCacheChange as Mail>::Result {
        if let Some(w) = self
            .stateful_writer_list
            .iter_mut()
            .find(|dw| dw.guid() == message.writer)
        {
            w.add_change(message.cache_change, &self.message_sender);
            w.send_message(&self.message_sender);
        }
    }
}

pub struct IsChangeAcknowledged {
    pub guid: Guid,
    pub sequence_number: SequenceNumber,
}
impl Mail for IsChangeAcknowledged {
    type Result = bool;
}
impl MailHandler<IsChangeAcknowledged> for RtpsParticipant {
    fn handle(&mut self, message: IsChangeAcknowledged) -> <IsChangeAcknowledged as Mail>::Result {
        if let Some(w) = self
            .stateful_writer_list
            .iter_mut()
            .find(|dw| dw.guid() == message.guid)
        {
            w.is_change_acknowledged(message.sequence_number)
        } else {
            false
        }
    }
}

pub struct IsHistoricalDataReceived {
    pub guid: Guid,
}
impl Mail for IsHistoricalDataReceived {
    type Result = bool;
}
impl MailHandler<IsHistoricalDataReceived> for RtpsParticipant {
    fn handle(
        &mut self,
        message: IsHistoricalDataReceived,
    ) -> <IsHistoricalDataReceived as Mail>::Result {
        if let Some(r) = self
            .stateful_reader_list
            .iter_mut()
            .find(|dw| dw.guid() == message.guid)
        {
            r.is_historical_data_received()
        } else {
            false
        }
    }
}
