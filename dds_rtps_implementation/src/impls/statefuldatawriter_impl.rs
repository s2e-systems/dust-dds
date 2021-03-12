use std::ops::{Deref, DerefMut};

use rust_dds_api::{dds_type::DDSType, publication::data_writer_listener::DataWriterListener};
use rust_rtps::{
    behavior::RTPSWriter,
    structure::{RTPSEndpoint, RTPSEntity},
};

struct RtpsDataWriterListener<T: DDSType>(Box<dyn DataWriterListener<DataType = T>>);
trait AnyDataWriterListener: Send + Sync {}

impl<T: DDSType> AnyDataWriterListener for RtpsDataWriterListener<T> {}

// pub enum RtpsWriterFlavor {
//     Stateful(StatefulWriter),
//     Stateless(StatelessWriter),
// }

// impl RtpsWriterFlavor {
//     fn produce_messages(&mut self, writer: &Writer) -> Vec<DestinedMessages> {
//         let writer_cache = &writer.writer_cache;
//         let entity_id = writer.endpoint.entity.guid.entity_id();
//         let last_change_sequence_number = writer.last_change_sequence_number;
//         let heartbeat_period = writer.heartbeat_period;
//         let nack_response_delay = writer.nack_response_delay;

//         let mut output = Vec::new();

//         match self {
//             RtpsWriterFlavor::Stateful(stateful_writer) => {
//                 for reader_proxy in stateful_writer.matched_readers.iter_mut() {
//                     let messages = ReliableReaderProxyBehavior::produce_messages(
//                         reader_proxy,
//                         writer_cache,
//                         entity_id,
//                         last_change_sequence_number,
//                         heartbeat_period,
//                         nack_response_delay,
//                     );
//                     output.push(DestinedMessages::MultiDestination {
//                         unicast_locator_list: reader_proxy.unicast_locator_list.clone(),
//                         multicast_locator_list: reader_proxy.multicast_locator_list.clone(),
//                         messages,
//                     });
//                 }
//             }
//             RtpsWriterFlavor::Stateless(stateless_writer) => {
//                 for reader_locator in stateless_writer.reader_locators.iter_mut() {
//                     let messages = BestEffortReaderLocatorBehavior::produce_messages(
//                         reader_locator,
//                         &writer.writer_cache,
//                         writer.endpoint.entity.guid.entity_id(),
//                         writer.last_change_sequence_number,
//                     );
//                     output.push(DestinedMessages::SingleDestination {
//                         locator: reader_locator.locator,
//                         messages,
//                     });
//                 }
//             }
//         }
//         output
//     }
// }

pub struct StatefulDataWriterImpl<W: RTPSWriter> {
    writer: W,
}

impl<W: RTPSWriter> StatefulDataWriterImpl<W> {
    // pub fn new<T: DDSType>(
    //     topic: Arc<Mutex<TopicImpl>>,
    //     qos: DataWriterQos,
    //     listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
    //     status_mask: StatusMask,
    // ) -> Self {
    //     let listener: Option<Box<dyn AnyDataWriterListener>> = match listener {
    //         Some(listener) => Some(Box::new(RtpsDataWriterListener(listener))),
    //         None => None,
    //     };
    //     let mask_listener = MaskListener::new(listener, status_mask);
    //     Self {
    //         qos,
    //         mask_listener,
    //         topic,
    //     }
    // }

    // pub fn produce_messages(&mut self) -> Vec<DestinedMessages> {
    //     let writer = &self.writer;
    //     let writer_cache = &writer.writer_cache;
    //     let entity_id = writer.endpoint.entity.guid.entity_id();
    //     let last_change_sequence_number = writer.last_change_sequence_number;
    //     let heartbeat_period = writer.heartbeat_period;
    //     let nack_response_delay = writer.nack_response_delay;

    //     let mut output = Vec::new();

    //     match &mut self.rtps_writer_flavor {
    //         RtpsWriterFlavor::Stateful(stateful_writer) => {
    //             for reader_proxy in stateful_writer.matched_readers.iter_mut() {
    //                 let messages = ReliableReaderProxyBehavior::produce_messages(
    //                     reader_proxy,
    //                     writer_cache,
    //                     entity_id,
    //                     last_change_sequence_number,
    //                     heartbeat_period,
    //                     nack_response_delay,
    //                 );
    //                 output.push(DestinedMessages::MultiDestination {
    //                     unicast_locator_list: reader_proxy.unicast_locator_list.clone(),
    //                     multicast_locator_list: reader_proxy.multicast_locator_list.clone(),
    //                     messages,
    //                 });
    //             }
    //         }
    //         RtpsWriterFlavor::Stateless(stateless_writer) => {
    //             for reader_locator in stateless_writer.reader_locators.iter_mut() {
    //                 let messages = BestEffortReaderLocatorBehavior::produce_messages(
    //                     reader_locator,
    //                     &writer.writer_cache,
    //                     writer.endpoint.entity.guid.entity_id(),
    //                     writer.last_change_sequence_number,
    //                 );
    //                 output.push(DestinedMessages::SingleDestination {
    //                     locator: reader_locator.locator,
    //                     messages,
    //                 });
    //             }
    //         }
    //     }
    //     output
    // }
}

impl<W: RTPSWriter> RTPSEntity for StatefulDataWriterImpl<W> {
    fn guid(&self) -> rust_rtps::types::GUID {
        todo!()
    }
}

impl<W: RTPSWriter> RTPSEndpoint for StatefulDataWriterImpl<W> {
    fn unicast_locator_list(&self) -> &[rust_rtps::types::Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[rust_rtps::types::Locator] {
        todo!()
    }

    fn topic_kind(&self) -> rust_rtps::types::TopicKind {
        todo!()
    }

    fn reliability_level(&self) -> rust_rtps::types::ReliabilityKind {
        todo!()
    }
}

impl<W: RTPSWriter> Deref for StatefulDataWriterImpl<W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<W: RTPSWriter> DerefMut for StatefulDataWriterImpl<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

// impl StatefulWriter<WriterImpl> for StatefulDataWriterImpl {
//     type ReaderProxyType = ReaderProxyImpl;

//     fn matched_readers(&self) -> &[Self::ReaderProxyType] {
//         todo!()
//     }

//     fn matched_reader_add(&mut self, _a_reader_proxy: Self::ReaderProxyType) {
//         todo!()
//     }

//     fn matched_reader_remove(&mut self, _reader_proxy_guid: &rust_rtps::types::GUID) {
//         todo!()
//     }

//     fn matched_reader_lookup(
//         &self,
//         _a_reader_guid: rust_rtps::types::GUID,
//     ) -> Option<&Self::ReaderProxyType> {
//         todo!()
//     }

//     fn is_acked_by_all(&self) -> bool {
//         todo!()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use core::panic;

//     use rust_rtps::{
//         behavior::{
//             stateless_writer::ReaderLocator,
//             types::{constants::DURATION_ZERO, Duration},
//         },
//         types::{
//             constants::ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER, ChangeKind, Locator,
//             ReliabilityKind, TopicKind, GUID,
//         },
//     };

//     use super::*;
//     #[test]
//     fn test() {
//         let stateless_writer = StatelessWriter::new();
//         let mut flavor = RtpsWriterFlavor::Stateless(stateless_writer);
//         let mut writer = Writer::new(
//             GUID::new([0; 12], ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER),
//             vec![],
//             vec![],
//             TopicKind::WithKey,
//             ReliabilityKind::BestEffort,
//             true,
//             DURATION_ZERO,
//             Duration::from_millis(200),
//             DURATION_ZERO,
//             None,
//         );
//         let messages = flavor.produce_messages(&writer);

//         assert_eq!(messages.len(), 0);

//         let cache_change = writer.new_change(ChangeKind::Alive, Some(vec![1, 2, 3]), None, [1; 16]);
//         writer.writer_cache.add_change(cache_change);

//         let mut stateless_writer = StatelessWriter::new();
//         let locator_expected = Locator::new_udpv4(1000, [1, 2, 3, 4]);
//         stateless_writer.reader_locator_add(ReaderLocator::new(locator_expected));
//         let mut flavor = RtpsWriterFlavor::Stateless(stateless_writer);
//         let messages_result = flavor.produce_messages(&writer);
//         assert_eq!(messages_result.len(), 1);

//         match &messages_result[0] {
//             DestinedMessages::SingleDestination {
//                 locator: locator_result,
//                 messages: _,
//             } => {
//                 assert_eq!(locator_result, &locator_expected);
//             }
//             _ => {
//                 panic!()
//             }
//         }
//     }
// }
