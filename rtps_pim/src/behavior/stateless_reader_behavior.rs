use crate::{
    messages::{
        submessage_elements::{
            EntityIdSubmessageElementAttributes, Parameter,
            ParameterListSubmessageElementAttributes, SequenceNumberSubmessageElementAttributes,
            SerializedDataSubmessageElementAttributes,
        },
        submessages::DataSubmessageAttributes,
    },
    structure::{
        cache_change::RtpsCacheChangeConstructor,
        history_cache::RtpsHistoryCacheOperations,
        types::{ChangeKind, Guid, GuidPrefix, ENTITYID_UNKNOWN},
    },
};

pub struct BestEffortStatelessReaderBehavior<'a, H> {
    pub reader_guid: Guid,
    pub reader_cache: &'a mut H,
}

impl<'a, H> BestEffortStatelessReaderBehavior<'a, H> {
    pub fn receive_data(
        &mut self,
        source_guid_prefix: GuidPrefix,
        data: &impl DataSubmessageAttributes<
            EntityIdSubmessageElementType = impl EntityIdSubmessageElementAttributes,
            SequenceNumberSubmessageElementType = impl SequenceNumberSubmessageElementAttributes,
            SerializedDataSubmessageElementType = impl SerializedDataSubmessageElementAttributes,
            ParameterListSubmessageElementType = impl ParameterListSubmessageElementAttributes,
        >,
    ) where
        H: RtpsHistoryCacheOperations,
        for<'b> H::CacheChangeType:
            RtpsCacheChangeConstructor<'b, DataType = &'b [u8], ParameterListType = &'b [Parameter<'b>]>,
    {
        let reader_id = data.reader_id().value();
        if reader_id == self.reader_guid.entity_id() || reader_id == ENTITYID_UNKNOWN {
            let kind = match (data.data_flag(), data.key_flag()) {
                (true, false) => ChangeKind::Alive,
                (false, true) => ChangeKind::NotAliveDisposed,
                _ => todo!(),
            };
            let writer_guid = Guid::new(source_guid_prefix, data.writer_id().value());
            let instance_handle = 0;
            let sequence_number = data.writer_sn().value();
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
            self.reader_cache.add_change(a_change);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        discovery::{
            sedp::builtin_endpoints::ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            spdp::builtin_endpoints::{
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
            },
        },
        messages::{submessage_elements::Parameter, types::SubmessageFlag},
        structure::types::{EntityId, SequenceNumber},
    };

    use super::*;

    struct MockEntityId {
        value: EntityId,
    }

    impl EntityIdSubmessageElementAttributes for MockEntityId {
        fn value(&self) -> EntityId {
            self.value
        }
    }

    struct MockSequenceNumber {
        value: SequenceNumber,
    }

    impl SequenceNumberSubmessageElementAttributes for MockSequenceNumber {
        fn value(&self) -> SequenceNumber {
            self.value
        }
    }

    struct MockParameterList;

    impl ParameterListSubmessageElementAttributes for MockParameterList {
        fn parameter(&self) -> &[Parameter<'_>] {
            &[]
        }
    }

    struct MockSerializedData;

    impl SerializedDataSubmessageElementAttributes for MockSerializedData {
        fn value(&self) -> &[u8] {
            &[]
        }
    }

    struct MockDataSubmessage {
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        reader_id: MockEntityId,
        writer_id: MockEntityId,
        writer_sn: MockSequenceNumber,
        serialized_payload: MockSerializedData,
        inline_qos: MockParameterList,
    }

    impl DataSubmessageAttributes for MockDataSubmessage {
        type EntityIdSubmessageElementType = MockEntityId;
        type SequenceNumberSubmessageElementType = MockSequenceNumber;
        type ParameterListSubmessageElementType = MockParameterList;
        type SerializedDataSubmessageElementType = MockSerializedData;

        fn endianness_flag(&self) -> SubmessageFlag {
            todo!()
        }

        fn inline_qos_flag(&self) -> SubmessageFlag {
            todo!()
        }

        fn data_flag(&self) -> SubmessageFlag {
            self.data_flag
        }

        fn key_flag(&self) -> SubmessageFlag {
            self.key_flag
        }

        fn non_standard_payload_flag(&self) -> SubmessageFlag {
            todo!()
        }

        fn reader_id(&self) -> &Self::EntityIdSubmessageElementType {
            &self.reader_id
        }

        fn writer_id(&self) -> &Self::EntityIdSubmessageElementType {
            &self.writer_id
        }

        fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType {
            &self.writer_sn
        }

        fn inline_qos(&self) -> &Self::ParameterListSubmessageElementType {
            &self.inline_qos
        }

        fn serialized_payload(&self) -> &Self::SerializedDataSubmessageElementType {
            &self.serialized_payload
        }
    }

    struct MockCacheChange;

    impl<'a> RtpsCacheChangeConstructor<'a> for MockCacheChange {
        type DataType = &'a [u8];
        type ParameterListType = &'a [Parameter<'a>];

        fn new(
            _kind: ChangeKind,
            _writer_guid: Guid,
            _instance_handle: crate::structure::types::InstanceHandle,
            _sequence_number: SequenceNumber,
            _data_value: Self::DataType,
            _inline_qos: Self::ParameterListType,
        ) -> Self {
            Self
        }
    }

    #[test]
    fn best_effort_stateless_reader_receive_data_reader_id_unknown() {
        struct MockHistoryCache(bool);

        impl RtpsHistoryCacheOperations for MockHistoryCache {
            type CacheChangeType = MockCacheChange;
            fn add_change(&mut self, _change: Self::CacheChangeType) {
                self.0 = true;
            }

            fn remove_change<F>(&mut self, _f: F)
            where
                F: FnMut(&Self::CacheChangeType) -> bool,
            {
                todo!()
            }

            fn get_seq_num_min(&self) -> Option<SequenceNumber> {
                todo!()
            }

            fn get_seq_num_max(&self) -> Option<SequenceNumber> {
                todo!()
            }
        }
        let mut history_cache = MockHistoryCache(false);
        let mut stateless_reader_behavior = BestEffortStatelessReaderBehavior {
            reader_guid: Guid::new(
                GuidPrefix([1; 12]),
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            ),
            reader_cache: &mut history_cache,
        };
        // let data_submessage = DataSubmessage {
        //     endianness_flag: true,
        //     inline_qos_flag: true,
        //     data_flag: true,
        //     key_flag: false,
        //     non_standard_payload_flag: false,
        //     reader_id: EntityIdSubmessageElement {
        //         value: ENTITYID_UNKNOWN,
        //     },
        //     writer_id: EntityIdSubmessageElement {
        //         value: ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        //     },
        //     writer_sn: SequenceNumberSubmessageElement { value: 1 },
        //     inline_qos: ParameterListSubmessageElement { parameter: () },
        //     serialized_payload: SerializedDataSubmessageElement { value: () },
        // };
        stateless_reader_behavior.receive_data(
            GuidPrefix([2; 12]),
            &MockDataSubmessage {
                data_flag: true,
                key_flag: false,
                reader_id: MockEntityId {
                    value: ENTITYID_UNKNOWN,
                },
                writer_id: MockEntityId {
                    value: ENTITYID_UNKNOWN,
                },
                writer_sn: MockSequenceNumber { value: 1 },
                inline_qos: MockParameterList,
                serialized_payload: MockSerializedData,
            },
        );

        assert_eq!(history_cache.0, true);
    }

    #[test]
    fn best_effort_stateless_reader_receive_data_reader_id_same_as_receiver() {
        struct MockHistoryCache(bool);

        impl RtpsHistoryCacheOperations for MockHistoryCache {
            type CacheChangeType = MockCacheChange;
            fn add_change(&mut self, _change: Self::CacheChangeType) {
                self.0 = true;
            }

            fn remove_change<F>(&mut self, _f: F)
            where
                F: FnMut(&Self::CacheChangeType) -> bool,
            {
                todo!()
            }

            fn get_seq_num_min(&self) -> Option<SequenceNumber> {
                todo!()
            }

            fn get_seq_num_max(&self) -> Option<SequenceNumber> {
                todo!()
            }
        }
        let mut history_cache = MockHistoryCache(false);
        let mut stateless_reader_behavior = BestEffortStatelessReaderBehavior {
            reader_guid: Guid::new(
                GuidPrefix([1; 12]),
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            ),
            reader_cache: &mut history_cache,
        };
        // let data_submessage = DataSubmessage {
        //     endianness_flag: true,
        //     inline_qos_flag: true,
        //     data_flag: true,
        //     key_flag: false,
        //     non_standard_payload_flag: false,
        //     reader_id: EntityIdSubmessageElement {
        //         value: ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
        //     },
        //     writer_id: EntityIdSubmessageElement {
        //         value: ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        //     },
        //     writer_sn: SequenceNumberSubmessageElement { value: 1 },
        //     inline_qos: ParameterListSubmessageElement { parameter: () },
        //     serialized_payload: SerializedDataSubmessageElement { value: () },
        // };
        stateless_reader_behavior.receive_data(
            GuidPrefix([2; 12]),
            &MockDataSubmessage {
                data_flag: true,
                key_flag: false,
                reader_id: MockEntityId {
                    value: ENTITYID_UNKNOWN,
                },
                writer_id: MockEntityId {
                    value: ENTITYID_UNKNOWN,
                },
                writer_sn: MockSequenceNumber { value: 1 },
                inline_qos: MockParameterList,
                serialized_payload: MockSerializedData,
            },
        );

        assert_eq!(history_cache.0, true);
    }

    #[test]
    fn best_effort_stateless_reader_receive_data_reader_id_other_than_receiver() {
        struct MockHistoryCache(bool);

        impl RtpsHistoryCacheOperations for MockHistoryCache {
            type CacheChangeType = MockCacheChange;
            fn add_change(&mut self, _change: Self::CacheChangeType) {
                self.0 = true;
            }

            fn remove_change<F>(&mut self, _f: F)
            where
                F: FnMut(&Self::CacheChangeType) -> bool,
            {
                todo!()
            }

            fn get_seq_num_min(&self) -> Option<SequenceNumber> {
                todo!()
            }

            fn get_seq_num_max(&self) -> Option<SequenceNumber> {
                todo!()
            }
        }
        let mut history_cache = MockHistoryCache(false);
        let mut stateless_reader_behavior = BestEffortStatelessReaderBehavior {
            reader_guid: Guid::new(
                GuidPrefix([1; 12]),
                ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER,
            ),
            reader_cache: &mut history_cache,
        };
        // let data_submessage = DataSubmessage {
        //     endianness_flag: true,
        //     inline_qos_flag: true,
        //     data_flag: true,
        //     key_flag: false,
        //     non_standard_payload_flag: false,
        //     reader_id: EntityIdSubmessageElement {
        //         value: ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        //     },
        //     writer_id: EntityIdSubmessageElement {
        //         value: ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
        //     },
        //     writer_sn: SequenceNumberSubmessageElement { value: 1 },
        //     inline_qos: ParameterListSubmessageElement { parameter: () },
        //     serialized_payload: SerializedDataSubmessageElement { value: () },
        // };
        stateless_reader_behavior.receive_data(
            GuidPrefix([2; 12]),
            &MockDataSubmessage {
                data_flag: true,
                key_flag: false,
                reader_id: MockEntityId {
                    value: ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
                },
                writer_id: MockEntityId {
                    value: ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER,
                },
                writer_sn: MockSequenceNumber { value: 1 },
                inline_qos: MockParameterList,
                serialized_payload: MockSerializedData,
            },
        );

        assert_eq!(history_cache.0, false);
    }
}
