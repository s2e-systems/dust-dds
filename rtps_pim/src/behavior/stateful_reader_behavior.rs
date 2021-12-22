use crate::{
    messages::{
        submessage_elements::{
            EntityIdSubmessageElementAttributes, ParameterListSubmessageElementAttributes,
            SequenceNumberSubmessageElementAttributes, SerializedDataSubmessageElementAttributes,
        },
        submessages::{DataSubmessage, DataSubmessageAttributes},
    },
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        history_cache::RtpsHistoryCacheAddChange,
        types::{ChangeKind, EntityId, Guid, GuidPrefix},
    },
};

use super::reader::{
    stateful_reader::RtpsStatefulReaderOperations,
    writer_proxy::{RtpsWriterProxyAttributes, RtpsWriterProxyOperations},
};

pub enum StatefulReaderBehavior<'a, W, H> {
    BestEffort(BestEffortStatefulReaderBehavior),
    Reliable(ReliableStatefulReaderBehavior<'a, W, H>),
}

pub struct BestEffortStatefulReaderBehavior;

impl BestEffortStatefulReaderBehavior {
    pub fn receive_data<L, P>(
        stateful_reader: &impl RtpsStatefulReaderOperations<
            L,
            WriterProxyType = impl RtpsWriterProxyOperations,
        >,
        source_guid_prefix: GuidPrefix,
        data: &DataSubmessage<P, &[u8]>,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value); // writer_guid := {Receiver.SourceGuidPrefix, DATA.writerId};
        if let Some(writer_proxy) = stateful_reader.matched_writer_lookup(&writer_guid) {
            let _expected_seq_nem = writer_proxy.available_changes_max(); // expected_seq_num := writer_proxy.available_changes_max() + 1;
        }
    }
}

pub struct ReliableStatefulReaderBehavior<'a, W, H> {
    pub writer_proxy: &'a mut W,
    pub reader_cache: &'a mut H,
}

impl<'a, W, H> ReliableStatefulReaderBehavior<'a, W, H> {
    pub fn receive_data(
        &mut self,
        source_guid_prefix: GuidPrefix,
        data: &impl DataSubmessageAttributes<
            EntityIdSubmessageElementType = impl EntityIdSubmessageElementAttributes<
                EntityIdType = EntityId,
            >,
            SequenceNumberSubmessageElementType = impl SequenceNumberSubmessageElementAttributes,
            SerializedDataSubmessageElementType = impl SerializedDataSubmessageElementAttributes<
                SerializedDataType = <H::CacheChangeType as RtpsCacheChangeConstructor>::DataType,
            >,
            ParameterListSubmessageElementType = impl ParameterListSubmessageElementAttributes,
        >,
    ) where
        W: RtpsWriterProxyAttributes + RtpsWriterProxyOperations,
        H: RtpsHistoryCacheAddChange,
        H::CacheChangeType: RtpsCacheChangeConstructor + RtpsCacheChangeAttributes,
    {
        let writer_guid = Guid::new(source_guid_prefix, *data.writer_id().value());
        if &writer_guid == self.writer_proxy.remote_writer_guid() {
            let kind = match (data.data_flag(), data.key_flag()) {
                (true, false) => ChangeKind::Alive,
                (false, true) => ChangeKind::NotAliveDisposed,
                _ => todo!(),
            };
            let instance_handle = 0;
            let sequence_number = *data.writer_sn().value();
            let data_value = data.serialized_payload().value();
            let inline_qos = data.inline_qos().parameter();
            let a_change = H::CacheChangeType::new(
                kind,
                writer_guid,
                instance_handle,
                sequence_number,
                data_value,
                inline_qos,
            );
            self.writer_proxy
                .received_change_set(a_change.sequence_number());
            self.reader_cache.add_change(a_change);
        }
    }

    pub fn send_ack_nack(&mut self) {
        todo!("ReliableStatefulReaderBehavior send AckNack");
    }

    pub fn receive_heartbeat(&mut self) {
        todo!("ReliableStatefulReaderBehavior send AckNack");
    }

    pub fn receive_gap(&mut self) {
        todo!("ReliableStatefulReaderBehavior receive Gap");
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::types::SubmessageFlag,
        structure::types::{EntityId, SequenceNumber},
    };

    use super::*;

    #[test]
    fn reliable_stateful_reader_receive_data() {
        struct MockWriterProxy(Guid);

        impl RtpsWriterProxyAttributes for MockWriterProxy {
            fn remote_writer_guid(&self) -> &Guid {
                &self.0
            }
        }

        impl RtpsWriterProxyOperations for MockWriterProxy {
            type SequenceNumberVector = ();

            fn available_changes_max(&self) -> &SequenceNumber {
                todo!()
            }

            fn irrelevant_change_set(&mut self, _a_seq_num: &SequenceNumber) {
                todo!()
            }

            fn lost_changes_update(&mut self, _first_available_seq_num: &SequenceNumber) {
                todo!()
            }

            fn missing_changes(&self) -> Self::SequenceNumberVector {
                todo!()
            }

            fn missing_changes_update(&mut self, _last_available_seq_num: &SequenceNumber) {
                todo!()
            }

            fn received_change_set(&mut self, a_seq_num: &SequenceNumber) {
                assert_eq!(a_seq_num, &1)
            }
        }

        struct MockCacheChange;

        impl RtpsCacheChangeConstructor for MockCacheChange {
            type DataType = ();
            type ParameterListType = ();

            fn new(
                kind: ChangeKind,
                writer_guid: Guid,
                instance_handle: crate::structure::types::InstanceHandle,
                sequence_number: SequenceNumber,
                data_value: Self::DataType,
                inline_qos: Self::ParameterListType,
            ) -> Self {
                Self
            }
        }

        impl RtpsCacheChangeAttributes for MockCacheChange {
            type DataType = ();
            type ParameterListType = ();

            fn kind(&self) -> &ChangeKind {
                todo!()
            }

            fn writer_guid(&self) -> &Guid {
                todo!()
            }

            fn instance_handle(&self) -> &crate::structure::types::InstanceHandle {
                todo!()
            }

            fn sequence_number(&self) -> &SequenceNumber {
                todo!()
            }

            fn data_value(&self) -> &Self::DataType {
                todo!()
            }

            fn inline_qos(&self) -> &Self::ParameterListType {
                todo!()
            }
        }

        struct MockReaderCache {
            add_change_called: bool,
        }

        impl RtpsHistoryCacheAddChange for MockReaderCache {
            type CacheChangeType = MockCacheChange;

            fn add_change(&mut self, _change: Self::CacheChangeType) {
                self.add_change_called = true;
            }
        }

        struct MockEntityId;

        impl EntityIdSubmessageElementAttributes for MockEntityId {
            type EntityIdType = EntityId;
            fn value(&self) -> &Self::EntityIdType {
                todo!()
            }
        }

        struct MockSequenceNumber;

        impl SequenceNumberSubmessageElementAttributes for MockSequenceNumber {
            fn value(&self) -> &SequenceNumber {
                todo!()
            }
        }

        struct MockParameterList;

        impl ParameterListSubmessageElementAttributes for MockParameterList {
            type ParameterListType = ();
            fn parameter(&self) -> &() {
                todo!()
            }
        }

        struct MockSerializedData;

        impl SerializedDataSubmessageElementAttributes for MockSerializedData {
            type SerializedDataType = ();
            fn value(&self) -> &Self::SerializedDataType {
                todo!()
            }
        }

        struct MockDataSubmessage;

        impl DataSubmessageAttributes for MockDataSubmessage {
            type EntityIdSubmessageElementType = MockEntityId;
            type SequenceNumberSubmessageElementType = MockSequenceNumber;
            type ParameterListSubmessageElementType = MockParameterList;
            type SerializedDataSubmessageElementType = MockSerializedData;

            fn endianness_flag(&self) -> &SubmessageFlag {
                todo!()
            }

            fn inline_qos_flag(&self) -> &SubmessageFlag {
                todo!()
            }

            fn data_flag(&self) -> &SubmessageFlag {
                todo!()
            }

            fn key_flag(&self) -> &SubmessageFlag {
                todo!()
            }

            fn non_standard_payload_flag(&self) -> &SubmessageFlag {
                todo!()
            }

            fn reader_id(&self) -> &Self::EntityIdSubmessageElementType {
                todo!()
            }

            fn writer_id(&self) -> &Self::EntityIdSubmessageElementType {
                todo!()
            }

            fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType {
                todo!()
            }

            fn inline_qos(&self) -> &Self::ParameterListSubmessageElementType {
                todo!()
            }

            fn serialized_payload(&self) -> &Self::SerializedDataSubmessageElementType {
                todo!()
            }
        }

        let mut mock_reader_cache = MockReaderCache {
            add_change_called: false,
        };

        let mut reliable_stateful_reader = ReliableStatefulReaderBehavior {
            writer_proxy: &mut MockWriterProxy(Guid::new(
                GuidPrefix([1; 12]),
                EntityId {
                    entity_key: [1; 3],
                    entity_kind: 2,
                },
            )),
            reader_cache: &mut mock_reader_cache,
        };
        let source_guid_prefix = GuidPrefix([1; 12]);
        // let data = DataSubmessage {
        //     endianness_flag: false,
        //     inline_qos_flag: true,
        //     data_flag: true,
        //     key_flag: false,
        //     non_standard_payload_flag: false,
        //     reader_id: EntityIdSubmessageElement {
        //         value: EntityId {
        //             entity_key: [1; 3],
        //             entity_kind: 1,
        //         },
        //     },
        //     writer_id: EntityIdSubmessageElement {
        //         value: EntityId {
        //             entity_key: [1; 3],
        //             entity_kind: 2,
        //         },
        //     },
        //     writer_sn: SequenceNumberSubmessageElement { value: 1 },
        //     inline_qos: ParameterListSubmessageElement { parameter: () },
        //     serialized_payload: SerializedDataSubmessageElement { value: () },
        // };
        reliable_stateful_reader.receive_data(source_guid_prefix, &MockDataSubmessage);

        assert_eq!(mock_reader_cache.add_change_called, true);
    }
}
