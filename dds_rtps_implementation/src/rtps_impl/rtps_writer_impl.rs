use rust_rtps_pim::{
    behavior::{ types::Duration},
    structure::types::{Guid, Locator, ReliabilityKind, SequenceNumber, TopicKind},
};

use super::{
    rtps_reader_locator_impl::RtpsReaderLocatorImpl, rtps_reader_proxy_impl::RtpsReaderProxyImpl,
    rtps_writer_history_cache_impl::WriterHistoryCache,
};

pub struct RtpsWriterImpl {
    guid: Guid,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    data_max_size_serialized: Option<i32>,
    reader_locators: Vec<RtpsReaderLocatorImpl>,
    matched_readers: Vec<RtpsReaderProxyImpl>,
    writer_cache: WriterHistoryCache,
}

// impl RtpsWriterOperations for RtpsWriterImpl {
//     fn new(
//         guid: Guid,
//         topic_kind: TopicKind,
//         reliability_level: ReliabilityKind,
//         unicast_locator_list: &[Locator],
//         multicast_locator_list: &[Locator],
//         push_mode: bool,
//         heartbeat_period: Duration,
//         nack_response_delay: Duration,
//         nack_suppression_duration: Duration,
//         data_max_size_serialized: Option<i32>,
//     ) -> Self {
//         Self {
//             guid,
//             topic_kind,
//             reliability_level,
//             push_mode,
//             unicast_locator_list: unicast_locator_list.into_iter().cloned().collect(),
//             multicast_locator_list: multicast_locator_list.into_iter().cloned().collect(),
//             heartbeat_period,
//             nack_response_delay,
//             nack_suppression_duration,
//             data_max_size_serialized,
//             last_change_sequence_number: 0.into(),
//             reader_locators: Vec::new(),
//             matched_readers: Vec::new(),
//             writer_cache: WriterHistoryCache::new(),
//         }
//     }

//     fn new_change<'a, C>(
//         &mut self,
//         kind: ChangeKind,
//         data: C::CacheChangeDataType,
//         inline_qos: &'a [Parameter<'a>],
//         handle: InstanceHandle,
//     ) -> RtpsCacheChange<'a, C::CacheChangeDataType>
//     where
//         C: RtpsHistoryCache<'a>,
//     {
//         self.last_change_sequence_number = self.last_change_sequence_number + 1;
//         RtpsCacheChange {
//             kind,
//             writer_guid: self.guid,
//             instance_handle: handle,
//             sequence_number: self.last_change_sequence_number,
//             data_value: data,
//             inline_qos,
//         }
//     }
// }

// impl RtpsStatelessWriterOperations for RtpsWriterImpl {
//     fn new(
//         guid: Guid,
//         topic_kind: TopicKind,
//         reliability_level: ReliabilityKind,
//         unicast_locator_list: &[Locator],
//         multicast_locator_list: &[Locator],
//         push_mode: bool,
//         heartbeat_period: Duration,
//         nack_response_delay: Duration,
//         nack_suppression_duration: Duration,
//         data_max_size_serialized: Option<i32>,
//     ) -> Self {
//         <Self as RtpsWriterOperations>::new(
//             guid,
//             topic_kind,
//             reliability_level,
//             unicast_locator_list,
//             multicast_locator_list,
//             push_mode,
//             heartbeat_period,
//             nack_response_delay,
//             nack_suppression_duration,
//             data_max_size_serialized,
//         )
//     }

//     fn reader_locator_add(&mut self, a_locator: Locator) {
//         let a_locator = RtpsReaderLocatorImpl::new(a_locator, false);
//         self.reader_locators.push(a_locator);
//     }

//     fn reader_locator_remove(&mut self, a_locator: &Locator) {
//         self.reader_locators.retain(|x| x.locator() != a_locator)
//     }

//     fn unsent_changes_reset(&mut self) {
//         for reader_locator in &mut self.reader_locators {
//             reader_locator.unsent_changes_reset();
//         }
//     }
// }

// impl RtpsStatefulWriter for RtpsWriterImpl {
//     type ReaderProxyType = RtpsReaderProxyImpl;

//     fn matched_readers(&self) -> &[Self::ReaderProxyType] {
//         &self.matched_readers
//     }
// }

// impl RtpsStatefulWriterOperations for RtpsWriterImpl {
//     fn new(
//         guid: Guid,
//         topic_kind: TopicKind,
//         reliability_level: ReliabilityKind,
//         unicast_locator_list: &[Locator],
//         multicast_locator_list: &[Locator],
//         push_mode: bool,
//         heartbeat_period: Duration,
//         nack_response_delay: Duration,
//         nack_suppression_duration: Duration,
//         data_max_size_serialized: Option<i32>,
//     ) -> Self {
//         <Self as RtpsWriterOperations>::new(
//             guid,
//             topic_kind,
//             reliability_level,
//             unicast_locator_list,
//             multicast_locator_list,
//             push_mode,
//             heartbeat_period,
//             nack_response_delay,
//             nack_suppression_duration,
//             data_max_size_serialized,
//         )
//     }

//     fn matched_reader_add(&mut self, a_reader_proxy: <Self as RtpsStatefulWriter>::ReaderProxyType)
//     where
//         Self: RtpsStatefulWriter,
//     {
//         self.matched_readers.push(a_reader_proxy)
//     }

//     fn matched_reader_remove(&mut self, reader_proxy_guid: &Guid) {
//         self.matched_readers
//             .retain(|x| x.remote_reader_guid() != reader_proxy_guid)
//     }

//     fn matched_reader_lookup(
//         &self,
//         a_reader_guid: &Guid,
//     ) -> Option<&<Self as RtpsStatefulWriter>::ReaderProxyType> {
//         self.matched_readers
//             .iter()
//             .find(|&x| x.remote_reader_guid() == a_reader_guid)
//     }

//     fn is_acked_by_all(&self) -> bool {
//         todo!()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use rust_rtps_pim::{
//         behavior::{
//             types::DURATION_ZERO,
//             writer::{
//                 reader_proxy::RtpsReaderProxyOperations,
//                 stateful_writer::{RtpsStatefulWriter, RtpsStatefulWriterOperations},
//                 stateless_writer::RtpsStatelessWriterOperations,
//                 writer::RtpsWriterOperations,
//             },
//         },
//         structure::{
//             types::{
//                 ChangeKind, EntityId, Guid, GuidPrefix, Locator, ReliabilityKind, TopicKind,
//                 GUID_UNKNOWN, USER_DEFINED_READER_NO_KEY, USER_DEFINED_UNKNOWN,
//             },
//             RtpsHistoryCache,
//         },
//     };

//     use crate::rtps_impl::{
//         rtps_reader_proxy_impl::RtpsReaderProxyImpl,
//         rtps_writer_history_cache_impl::WriterHistoryCache,
//     };

//     use super::RtpsWriterImpl;

//     #[test]
//     fn new_change() {
//         let mut writer = RtpsWriterImpl {
//             guid: GUID_UNKNOWN,
//             topic_kind: TopicKind::WithKey,
//             reliability_level: ReliabilityKind::BestEffort,
//             push_mode: true,
//             unicast_locator_list: vec![],
//             multicast_locator_list: vec![],
//             heartbeat_period: DURATION_ZERO,
//             nack_response_delay: DURATION_ZERO,
//             nack_suppression_duration: DURATION_ZERO,
//             last_change_sequence_number: 0,
//             data_max_size_serialized: None,
//             reader_locators: Vec::new(),
//             matched_readers: Vec::new(),
//             writer_cache: WriterHistoryCache::new(),
//         };
//         let change1 = writer.new_change::<WriterHistoryCache>(ChangeKind::Alive, vec![], &[], 0);
//         let change2 = writer.new_change::<WriterHistoryCache>(ChangeKind::Alive, vec![], &[], 0);

//         assert_eq!(change1.sequence_number, 1);
//         assert_eq!(change2.sequence_number, 2);
//     }

//     #[test]
//     fn reader_locator_add() {
//         let mut writer = RtpsWriterImpl {
//             guid: GUID_UNKNOWN,
//             topic_kind: TopicKind::WithKey,
//             reliability_level: ReliabilityKind::BestEffort,
//             push_mode: true,
//             unicast_locator_list: vec![],
//             multicast_locator_list: vec![],
//             heartbeat_period: DURATION_ZERO,
//             nack_response_delay: DURATION_ZERO,
//             nack_suppression_duration: DURATION_ZERO,
//             last_change_sequence_number: 0,
//             data_max_size_serialized: None,
//             reader_locators: Vec::new(),
//             matched_readers: Vec::new(),
//             writer_cache: WriterHistoryCache::new(),
//         };
//         writer.reader_locator_add(Locator::new(1, 1, [1; 16]));
//         writer.reader_locator_add(Locator::new(2, 2, [2; 16]));

//         assert_eq!(writer.reader_locators.len(), 2)
//     }

//     #[test]
//     fn reader_locator_remove() {
//         let mut writer = RtpsWriterImpl {
//             guid: GUID_UNKNOWN,
//             topic_kind: TopicKind::WithKey,
//             reliability_level: ReliabilityKind::BestEffort,
//             push_mode: true,
//             unicast_locator_list: vec![],
//             multicast_locator_list: vec![],
//             heartbeat_period: DURATION_ZERO,
//             nack_response_delay: DURATION_ZERO,
//             nack_suppression_duration: DURATION_ZERO,
//             last_change_sequence_number: 0,
//             data_max_size_serialized: None,
//             reader_locators: Vec::new(),
//             matched_readers: Vec::new(),
//             writer_cache: WriterHistoryCache::new(),
//         };

//         writer.reader_locator_add(Locator::new(1, 1, [1; 16]));
//         writer.reader_locator_add(Locator::new(2, 2, [2; 16]));
//         writer.reader_locator_remove(&Locator::new(1, 1, [1; 16]));

//         assert_eq!(writer.reader_locators.len(), 1)
//     }

//     #[test]
//     fn matched_reader_add() {
//         let mut writer = RtpsWriterImpl {
//             guid: GUID_UNKNOWN,
//             topic_kind: TopicKind::WithKey,
//             reliability_level: ReliabilityKind::BestEffort,
//             push_mode: true,
//             unicast_locator_list: vec![],
//             multicast_locator_list: vec![],
//             heartbeat_period: DURATION_ZERO,
//             nack_response_delay: DURATION_ZERO,
//             nack_suppression_duration: DURATION_ZERO,
//             last_change_sequence_number: 0,
//             data_max_size_serialized: None,
//             reader_locators: Vec::new(),
//             matched_readers: Vec::new(),
//             writer_cache: WriterHistoryCache::new(),
//         };
//         let unknown_remote_group_entity_id = EntityId::new([0; 3], USER_DEFINED_UNKNOWN);
//         let reader_proxy_guid1 = Guid::new(
//             GuidPrefix([1; 12]),
//             EntityId::new([1; 3], USER_DEFINED_READER_NO_KEY),
//         );
//         let reader_proxy1 = RtpsReaderProxyImpl::new(
//             reader_proxy_guid1,
//             unknown_remote_group_entity_id,
//             &[],
//             &[],
//             false,
//             true,
//         );
//         let reader_proxy_guid2 = Guid::new(
//             GuidPrefix([2; 12]),
//             EntityId::new([2; 3], USER_DEFINED_READER_NO_KEY),
//         );
//         let reader_proxy2 = RtpsReaderProxyImpl::new(
//             reader_proxy_guid2,
//             unknown_remote_group_entity_id,
//             &[],
//             &[],
//             false,
//             true,
//         );
//         writer.matched_reader_add(reader_proxy1);
//         writer.matched_reader_add(reader_proxy2);
//         assert_eq!(writer.matched_readers().len(), 2)
//     }

//     #[test]
//     fn matched_reader_remove() {
//         let mut writer = RtpsWriterImpl {
//             guid: GUID_UNKNOWN,
//             topic_kind: TopicKind::WithKey,
//             reliability_level: ReliabilityKind::BestEffort,
//             push_mode: true,
//             unicast_locator_list: vec![],
//             multicast_locator_list: vec![],
//             heartbeat_period: DURATION_ZERO,
//             nack_response_delay: DURATION_ZERO,
//             nack_suppression_duration: DURATION_ZERO,
//             last_change_sequence_number: 0,
//             data_max_size_serialized: None,
//             reader_locators: Vec::new(),
//             matched_readers: Vec::new(),
//             writer_cache: WriterHistoryCache::new(),
//         };

//         let unknown_remote_group_entity_id = EntityId::new([0; 3], USER_DEFINED_UNKNOWN);
//         let reader_proxy_guid1 = Guid::new(
//             GuidPrefix([1; 12]),
//             EntityId::new([1; 3], USER_DEFINED_READER_NO_KEY),
//         );
//         let reader_proxy1 = RtpsReaderProxyImpl::new(
//             reader_proxy_guid1,
//             unknown_remote_group_entity_id,
//             &[],
//             &[],
//             false,
//             true,
//         );
//         let reader_proxy_guid2 = Guid::new(
//             GuidPrefix([2; 12]),
//             EntityId::new([2; 3], USER_DEFINED_READER_NO_KEY),
//         );
//         let reader_proxy2 = RtpsReaderProxyImpl::new(
//             reader_proxy_guid2,
//             unknown_remote_group_entity_id,
//             &[],
//             &[],
//             false,
//             true,
//         );
//         writer.matched_reader_add(reader_proxy1);
//         writer.matched_reader_add(reader_proxy2);
//         writer.matched_reader_remove(&reader_proxy_guid2);

//         assert_eq!(writer.matched_readers().len(), 1)
//     }

//     #[test]
//     fn matched_reader_lookup() {
//         let mut writer = RtpsWriterImpl {
//             guid: GUID_UNKNOWN,
//             topic_kind: TopicKind::WithKey,
//             reliability_level: ReliabilityKind::BestEffort,
//             push_mode: true,
//             unicast_locator_list: vec![],
//             multicast_locator_list: vec![],
//             heartbeat_period: DURATION_ZERO,
//             nack_response_delay: DURATION_ZERO,
//             nack_suppression_duration: DURATION_ZERO,
//             last_change_sequence_number: 0,
//             data_max_size_serialized: None,
//             reader_locators: Vec::new(),
//             matched_readers: Vec::new(),
//             writer_cache: WriterHistoryCache::new(),
//         };

//         let unknown_remote_group_entity_id = EntityId::new([0; 3], USER_DEFINED_UNKNOWN);
//         let reader_proxy_guid1 = Guid::new(
//             GuidPrefix([1; 12]),
//             EntityId::new([1; 3], USER_DEFINED_READER_NO_KEY),
//         );
//         let reader_proxy1 = RtpsReaderProxyImpl::new(
//             reader_proxy_guid1,
//             unknown_remote_group_entity_id,
//             &[],
//             &[],
//             false,
//             true,
//         );
//         let reader_proxy_guid2 = Guid::new(
//             GuidPrefix([2; 12]),
//             EntityId::new([2; 3], USER_DEFINED_READER_NO_KEY),
//         );
//         let reader_proxy2 = RtpsReaderProxyImpl::new(
//             reader_proxy_guid2,
//             unknown_remote_group_entity_id,
//             &[],
//             &[],
//             false,
//             true,
//         );
//         writer.matched_reader_add(reader_proxy1);
//         writer.matched_reader_add(reader_proxy2);

//         assert!(writer.matched_reader_lookup(&reader_proxy_guid1).is_some());
//         assert!(writer.matched_reader_lookup(&GUID_UNKNOWN).is_none());
//     }
// }
