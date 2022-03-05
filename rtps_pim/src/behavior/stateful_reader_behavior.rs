use core::iter::FromIterator;

use crate::{
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, Parameter,
            SequenceNumberSetSubmessageElement,
        },
        submessages::{AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage},
        types::Count,
    },
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        history_cache::RtpsHistoryCacheOperations,
        types::{ChangeKind, EntityId, Guid, GuidPrefix, SequenceNumber},
    },
};

use super::reader::writer_proxy::{RtpsWriterProxyAttributes, RtpsWriterProxyOperations};

pub struct BestEffortStatefulReaderBehavior;

impl BestEffortStatefulReaderBehavior {
    pub fn receive_data<'a, C, P>(
        writer_proxy: &mut impl RtpsWriterProxyOperations,
        reader_cache: &mut impl RtpsHistoryCacheOperations<CacheChangeType = C>,
        source_guid_prefix: GuidPrefix,
        data: &'a DataSubmessage<'_, P>,
    ) where
        C: RtpsCacheChangeConstructor<
                'a,
                DataType = &'a [u8],
                ParameterListType = &'a [Parameter<'a>],
            > + RtpsCacheChangeAttributes<'a>,
        P: AsRef<[Parameter<'a>]>,
    {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value);
        let kind = match (data.data_flag, data.key_flag) {
            (true, false) => ChangeKind::Alive,
            (false, true) => ChangeKind::NotAliveDisposed,
            _ => todo!(),
        };
        let instance_handle = 0;
        let sequence_number = data.writer_sn.value;
        let data_value = data.serialized_payload.value;
        let inline_qos = data.inline_qos.parameter.as_ref();
        let a_change: C = RtpsCacheChangeConstructor::new(
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data_value,
            inline_qos,
        );

        let expected_seq_num = writer_proxy.available_changes_max() + 1; // expected_seq_num := writer_proxy.available_changes_max() + 1;
        if a_change.sequence_number() >= expected_seq_num {
            writer_proxy.received_change_set(a_change.sequence_number());
            if a_change.sequence_number() > expected_seq_num {
                writer_proxy.lost_changes_update(a_change.sequence_number());
            }
            reader_cache.add_change(a_change);
        }
    }

    pub fn receive_gap<S>(writer_proxy: &mut impl RtpsWriterProxyOperations, gap: GapSubmessage<S>)
    where
        S: IntoIterator<Item = SequenceNumber>,
    {
        for seq_num in gap.gap_start.value..gap.gap_list.base - 1 {
            writer_proxy.irrelevant_change_set(seq_num);
        }
        for seq_num in gap.gap_list.set {
            writer_proxy.irrelevant_change_set(seq_num);
        }
    }
}

pub struct ReliableStatefulReaderBehavior;

impl ReliableStatefulReaderBehavior {
    pub fn receive_data<'a, C, P>(
        writer_proxy: &mut impl RtpsWriterProxyOperations,
        reader_cache: &mut impl RtpsHistoryCacheOperations<CacheChangeType = C>,
        source_guid_prefix: GuidPrefix,
        data: &'a DataSubmessage<'_, P>,
    ) where
        C: RtpsCacheChangeConstructor<
                'a,
                DataType = &'a [u8],
                ParameterListType = &'a [Parameter<'a>],
            > + RtpsCacheChangeAttributes<'a>,
        P: AsRef<[Parameter<'a>]>,
    {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value);

        let kind = match (data.data_flag, data.key_flag) {
            (true, false) => ChangeKind::Alive,
            (false, true) => ChangeKind::NotAliveDisposed,
            _ => todo!(),
        };
        let instance_handle = 0;
        let sequence_number = data.writer_sn.value;
        let data_value = data.serialized_payload.value;
        let inline_qos = data.inline_qos.parameter.as_ref();
        let a_change = C::new(
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data_value,
            inline_qos,
        );
        writer_proxy.received_change_set(a_change.sequence_number());
        reader_cache.add_change(a_change);
    }

    pub fn send_ack_nack<S>(
        writer_proxy: &mut (impl RtpsWriterProxyOperations + RtpsWriterProxyAttributes),
        reader_id: EntityId,
        acknack_count: Count,
        mut send_acknack: impl FnMut(AckNackSubmessage<S>),
    ) where
        S: FromIterator<SequenceNumber>,
    {
        let endianness_flag = true;
        let final_flag = false;
        let reader_id = EntityIdSubmessageElement { value: reader_id };
        let writer_id = EntityIdSubmessageElement {
            value: writer_proxy.remote_writer_guid().entity_id,
        };
        let reader_sn_state = SequenceNumberSetSubmessageElement {
            base: writer_proxy.available_changes_max() + 1,
            set: core::iter::empty().collect(), // FOREACH change IN the_writer_proxy.missing_changes() DO ADD change.sequenceNumber TO missing_seq_num_set.set;
        };
        let count = CountSubmessageElement {
            value: acknack_count,
        };

        let acknack_submessage = AckNackSubmessage {
            endianness_flag,
            final_flag,
            reader_id,
            writer_id,
            reader_sn_state,
            count,
        };

        send_acknack(acknack_submessage);
    }

    pub fn receive_heartbeat(
        writer_proxy: &mut impl RtpsWriterProxyOperations,
        heartbeat: HeartbeatSubmessage,
    ) {
        writer_proxy.missing_changes_update(heartbeat.last_sn.value);
        writer_proxy.lost_changes_update(heartbeat.first_sn.value);
    }

    pub fn receive_gap<S>(writer_proxy: &mut impl RtpsWriterProxyOperations, gap: GapSubmessage<S>)
    where
        S: IntoIterator<Item = SequenceNumber>,
    {
        for seq_num in gap.gap_start.value..gap.gap_list.base - 1 {
            writer_proxy.irrelevant_change_set(seq_num);
        }
        for seq_num in gap.gap_list.set {
            writer_proxy.irrelevant_change_set(seq_num);
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         messages::{submessage_elements::{Parameter, ParameterListSubmessageElement}, types::SubmessageFlag},
//         structure::types::{EntityId, InstanceHandle, SequenceNumber},
//     };

//     use super::*;

//     #[test]
//     fn reliable_stateful_reader_receive_data() {
//         struct MockWriterProxy(Guid);

//         impl RtpsWriterProxyAttributes for MockWriterProxy {
//             fn remote_writer_guid(&self) -> Guid {
//                 self.0
//             }

//             fn unicast_locator_list(&self) -> &[crate::structure::types::Locator] {
//                 todo!()
//             }

//             fn multicast_locator_list(&self) -> &[crate::structure::types::Locator] {
//                 todo!()
//             }

//             fn data_max_size_serialized(&self) -> Option<i32> {
//                 todo!()
//             }

//             fn remote_group_entity_id(&self) -> EntityId {
//                 todo!()
//             }
//         }

//         impl RtpsWriterProxyOperations for MockWriterProxy {
//             type SequenceNumberListType = ();

//             fn available_changes_max(&self) -> SequenceNumber {
//                 todo!()
//             }

//             fn irrelevant_change_set(&mut self, _a_seq_num: SequenceNumber) {
//                 todo!()
//             }

//             fn lost_changes_update(&mut self, _first_available_seq_num: SequenceNumber) {
//                 todo!()
//             }

//             fn missing_changes(&self) -> Self::SequenceNumberListType {
//                 todo!()
//             }

//             fn missing_changes_update(&mut self, _last_available_seq_num: SequenceNumber) {
//                 todo!()
//             }

//             fn received_change_set(&mut self, a_seq_num: SequenceNumber) {
//                 assert_eq!(a_seq_num, 1)
//             }
//         }

//         struct MockCacheChange {
//             sequence_number: SequenceNumber,
//         }

//         impl<'a> RtpsCacheChangeConstructor<'a> for MockCacheChange {
//             type DataType = &'a [u8];
//             type ParameterListType = &'a [Parameter<'a>];

//             fn new(
//                 _kind: ChangeKind,
//                 _writer_guid: Guid,
//                 _instance_handle: InstanceHandle,
//                 sequence_number: SequenceNumber,
//                 _data_value: Self::DataType,
//                 _inline_qos: Self::ParameterListType,
//             ) -> Self {
//                 Self {
//                     sequence_number,
//                 }
//             }
//         }

//         impl<'a> RtpsCacheChangeAttributes<'a> for MockCacheChange {
//             type DataType = ();
//             type ParameterListType = [Parameter<'a>];

//             fn kind(&self) -> ChangeKind {
//                 todo!()
//             }

//             fn writer_guid(&self) -> Guid {
//                 todo!()
//             }

//             fn instance_handle(&self) -> InstanceHandle {
//                 todo!()
//             }

//             fn sequence_number(&self) -> SequenceNumber {
//                 self.sequence_number
//             }

//             fn data_value(&self) -> &Self::DataType {
//                 todo!()
//             }

//             fn inline_qos(&self) -> &Self::ParameterListType {
//                 todo!()
//             }
//         }

//         struct MockReaderCache {
//             add_change_called: bool,
//         }

//         impl RtpsHistoryCacheOperations for MockReaderCache {
//             type CacheChangeType = MockCacheChange;

//             fn add_change(&mut self, _change: Self::CacheChangeType) {
//                 self.add_change_called = true;
//             }

//             fn remove_change<F>(&mut self, _f: F)
//             where
//                 F: FnMut(&Self::CacheChangeType) -> bool,
//             {
//                 todo!()
//             }

//             fn get_seq_num_min(&self) -> Option<SequenceNumber> {
//                 todo!()
//             }

//             fn get_seq_num_max(&self) -> Option<SequenceNumber> {
//                 todo!()
//             }
//         }

//         struct MockEntityId {
//             value: EntityId,
//         }

//         impl EntityIdSubmessageElementAttributes for MockEntityId {
//             fn value(&self) -> EntityId {
//                 self.value
//             }
//         }

//         struct MockSequenceNumber {
//             value: SequenceNumber,
//         }

//         impl SequenceNumberSubmessageElementAttributes for MockSequenceNumber {
//             fn value(&self) -> SequenceNumber {
//                 self.value
//             }
//         }

//         struct MockParameterList;

//         impl ParameterListSubmessageElementAttributes for MockParameterList {
//             fn parameter(&self) -> &[Parameter<'_>] {
//                 &[]
//             }
//         }

//         struct MockSerializedData;

//         impl SerializedDataSubmessageElementAttributes for MockSerializedData {
//             fn value(&self) -> &[u8] {
//                 &[]
//             }
//         }

//         struct MockDataSubmessage {
//             data_flag: SubmessageFlag,
//             key_flag: SubmessageFlag,
//             writer_id: MockEntityId,
//             writer_sn: MockSequenceNumber,
//             inline_qos: MockParameterList,
//             serialized_payload: MockSerializedData,
//         }

//         impl DataSubmessageAttributes<&Parameter<'_>> for MockDataSubmessage {
//             type EntityIdSubmessageElementType = MockEntityId;
//             type SequenceNumberSubmessageElementType = MockSequenceNumber;
//             type ParameterListSubmessageElementType = MockParameterList;
//             type SerializedDataSubmessageElementType = MockSerializedData;

//             fn endianness_flag(&self) -> SubmessageFlag {
//                 todo!()
//             }

//             fn inline_qos_flag(&self) -> SubmessageFlag {
//                 todo!()
//             }

//             fn data_flag(&self) -> SubmessageFlag {
//                 self.data_flag
//             }

//             fn key_flag(&self) -> SubmessageFlag {
//                 self.key_flag
//             }

//             fn non_standard_payload_flag(&self) -> SubmessageFlag {
//                 todo!()
//             }

//             fn reader_id(&self) -> &Self::EntityIdSubmessageElementType {
//                 todo!()
//             }

//             fn writer_id(&self) -> &Self::EntityIdSubmessageElementType {
//                 &self.writer_id
//             }

//             fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType {
//                 &self.writer_sn
//             }

//             fn inline_qos(&self) -> &ParameterListSubmessageElement<&[Parameter<'_>]> {
//                 // &self.inline_qos
//                 todo!()
//             }

//             fn serialized_payload(&self) -> &Self::SerializedDataSubmessageElementType {
//                 &self.serialized_payload
//             }
//         }

//         let mut mock_reader_cache = MockReaderCache {
//             add_change_called: false,
//         };

//         let mut reliable_stateful_reader = ReliableStatefulReaderBehavior {
//             writer_proxy: &mut MockWriterProxy(Guid::new(
//                 GuidPrefix([1; 12]),
//                 EntityId {
//                     entity_key: [1; 3],
//                     entity_kind: 2,
//                 },
//             )),
//             reader_cache: &mut mock_reader_cache,
//         };
//         let source_guid_prefix = GuidPrefix([1; 12]);
//         // let data = DataSubmessage {
//         //     endianness_flag: false,
//         //     inline_qos_flag: true,
//         //     data_flag: true,
//         //     key_flag: false,
//         //     non_standard_payload_flag: false,
//         //     reader_id: EntityIdSubmessageElement {
//         //         value: EntityId {
//         //             entity_key: [1; 3],
//         //             entity_kind: 1,
//         //         },
//         //     },
//         //     writer_id: EntityIdSubmessageElement {
//         //         value: EntityId {
//         //             entity_key: [1; 3],
//         //             entity_kind: 2,
//         //         },
//         //     },
//         //     writer_sn: SequenceNumberSubmessageElement { value: 1 },
//         //     inline_qos: ParameterListSubmessageElement { parameter: () },
//         //     serialized_payload: SerializedDataSubmessageElement { value: () },
//         // };
//         reliable_stateful_reader.receive_data(
//             source_guid_prefix,
//             &MockDataSubmessage {
//                 data_flag: true,
//                 key_flag: false,
//                 writer_id: MockEntityId {
//                     value: EntityId {
//                         entity_key: [1; 3],
//                         entity_kind: 2,
//                     },
//                 },
//                 writer_sn: MockSequenceNumber { value: 1 },
//                 inline_qos: MockParameterList,
//                 serialized_payload: MockSerializedData,
//             },
//         );

//         assert_eq!(mock_reader_cache.add_change_called, true);
//     }
// }
