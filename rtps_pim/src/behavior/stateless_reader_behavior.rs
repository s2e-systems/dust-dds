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
        cache_change::RtpsCacheChange,
        history_cache::RtpsHistoryCacheAddChange,
        types::{ChangeKind, EntityId, Guid, GuidPrefix, ENTITYID_UNKNOWN},
    },
};

pub struct BestEffortStatelessReaderBehavior<'a, C> {
    pub reader_guid: &'a Guid,
    pub reader_cache: &'a mut C,
}

impl<C> BestEffortStatelessReaderBehavior<'_, C> {
    pub fn receive_data(
        &mut self,
        source_guid_prefix: GuidPrefix,
        data: &impl DataSubmessageAttributes<
            EntityIdSubmessageElementType = impl EntityIdSubmessageElementAttributes<
                EntityIdType = EntityId,
            >,
            SequenceNumberSubmessageElementType = impl SequenceNumberSubmessageElementAttributes,
            ParameterListSubmessageElementType = impl ParameterListSubmessageElementAttributes,
            SerializedDataSubmessageElementType = impl SerializedDataSubmessageElementAttributes,
        >,
    ) where
        C: for<'a> RtpsHistoryCacheAddChange<
            'a,
            ParameterListType = &'a [Parameter<&'a [u8]>],
            DataType = &'a [u8],
        >,
    {
        let reader_id = data.reader_id().value();
        if reader_id == self.reader_guid.entity_id() || reader_id == &ENTITYID_UNKNOWN {
            let kind = match (data.data_flag(), data.key_flag()) {
                (true, false) => ChangeKind::Alive,
                (false, true) => ChangeKind::NotAliveDisposed,
                _ => todo!(),
            };
            let writer_guid = Guid::new(source_guid_prefix, *data.writer_id().value());
            let instance_handle = 0;
            let sequence_number = *data.writer_sn().value();
            let data_value = data.serialized_payload().value();
            let inline_qos = data.inline_qos().parameter();
            let a_change = RtpsCacheChange {
                kind,
                writer_guid,
                instance_handle,
                sequence_number,
                data_value,
                inline_qos,
            };
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
        messages::submessage_elements::{
            EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
        },
        structure::types::{EntityId, SequenceNumber},
    };

    use super::*;

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
        fn parameter(&self) -> &[Parameter<&[u8]>] {
            todo!()
        }
    }

    struct MockSerializedData;

    impl SerializedDataSubmessageElementAttributes for MockSerializedData {
        fn value(&self) -> &[u8] {
            todo!()
        }
    }

    struct MockDataSubmessage;

    impl DataSubmessageAttributes for MockDataSubmessage {
        type EntityIdSubmessageElementType = MockEntityId;
        type SequenceNumberSubmessageElementType = MockSequenceNumber;
        type ParameterListSubmessageElementType = MockParameterList;
        type SerializedDataSubmessageElementType = MockSerializedData;

        fn endianness_flag(&self) -> &crate::messages::types::SubmessageFlag {
            todo!()
        }

        fn inline_qos_flag(&self) -> &crate::messages::types::SubmessageFlag {
            todo!()
        }

        fn data_flag(&self) -> &crate::messages::types::SubmessageFlag {
            todo!()
        }

        fn key_flag(&self) -> &crate::messages::types::SubmessageFlag {
            todo!()
        }

        fn non_standard_payload_flag(&self) -> &crate::messages::types::SubmessageFlag {
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

    #[test]
    fn best_effort_stateless_reader_receive_data_reader_id_unknown() {
        struct MockHistoryCache(bool);

        impl<'a> RtpsHistoryCacheAddChange<'a> for MockHistoryCache {
            type ParameterListType = &'a [Parameter<&'a [u8]>];
            type DataType = &'a [u8];

            fn add_change(
                &mut self,
                _change: RtpsCacheChange<Self::ParameterListType, Self::DataType>,
            ) {
                self.0 = true;
            }
        }
        let mut history_cache = MockHistoryCache(false);
        let mut stateless_reader_behavior = BestEffortStatelessReaderBehavior {
            reader_guid: &Guid::new(
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
        stateless_reader_behavior.receive_data(GuidPrefix([2; 12]), &MockDataSubmessage);

        assert_eq!(history_cache.0, true);
    }

    #[test]
    fn best_effort_stateless_reader_receive_data_reader_id_same_as_receiver() {
        struct MockHistoryCache(bool);

        impl<'a> RtpsHistoryCacheAddChange<'a> for MockHistoryCache {
            type ParameterListType = &'a [Parameter<&'a [u8]>];
            type DataType = &'a [u8];

            fn add_change(
                &mut self,
                _change: RtpsCacheChange<Self::ParameterListType, Self::DataType>,
            ) {
                self.0 = true;
            }
        }
        let mut history_cache = MockHistoryCache(false);
        let mut stateless_reader_behavior = BestEffortStatelessReaderBehavior {
            reader_guid: &Guid::new(
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
        stateless_reader_behavior.receive_data(GuidPrefix([2; 12]), &MockDataSubmessage);

        assert_eq!(history_cache.0, true);
    }

    #[test]
    fn best_effort_stateless_reader_receive_data_reader_id_other_than_receiver() {
        struct MockHistoryCache(bool);

        impl<'a> RtpsHistoryCacheAddChange<'a> for MockHistoryCache {
            type ParameterListType = &'a [Parameter<&'a [u8]>];
            type DataType = &'a [u8];

            fn add_change(
                &mut self,
                _change: RtpsCacheChange<Self::ParameterListType, Self::DataType>,
            ) {
                self.0 = true;
            }
        }
        let mut history_cache = MockHistoryCache(false);
        let mut stateless_reader_behavior = BestEffortStatelessReaderBehavior {
            reader_guid: &Guid::new(
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
        stateless_reader_behavior.receive_data(GuidPrefix([2; 12]), &MockDataSubmessage);

        assert_eq!(history_cache.0, false);
    }
}
