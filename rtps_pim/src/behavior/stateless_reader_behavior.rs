use crate::{
    messages::{
        submessage_elements::{
            EntityIdSubmessageElementType, SequenceNumberSubmessageElementType,
            SerializedDataSubmessageElementType,
        },
        submessages::DataSubmessage,
    },
    structure::{
        types::{ChangeKind, GuidPrefix, ENTITYID_UNKNOWN, GUID},
        RTPSCacheChangeOperations, RTPSEntity, RTPSHistoryCache,
    },
};

use super::reader::reader::RTPSReader;

pub trait StatelessReaderBehavior<Data> {
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &Data);
}

impl<'a, 'b, T, Data> StatelessReaderBehavior<Data> for T
where
    T: RTPSReader + RTPSEntity,
    T::HistoryCacheType: RTPSHistoryCache,
    <T::HistoryCacheType as RTPSHistoryCache>::CacheChange: RTPSCacheChangeOperations<
        'b,
        InstanceHandleType = i32,
        DataType = &'b [u8],
        InlineQosType = (),
    >,
    Data: DataSubmessage<'b>,
{
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &Data) {
        let reader_id = data.reader_id().value();
        if &reader_id == self.guid().entity_id() || reader_id == ENTITYID_UNKNOWN {
            let reader_cache = self.reader_cache_mut();
            let kind = match (data.data_flag(), data.key_flag()) {
                (true, false) => ChangeKind::Alive,
                (false, true) => ChangeKind::NotAliveDisposed,
                _ => todo!(),
            };
            let writer_guid = GUID::new(source_guid_prefix, data.writer_id().value());
            let instance_handle = 0;
            let sequence_number = data.writer_sn().value();
            let data = data.serialized_payload().value();
            let inline_qos = ();
            let a_change = RTPSCacheChangeOperations::new(
                kind,
                writer_guid,
                instance_handle,
                sequence_number,
                data,
                inline_qos,
            );
            reader_cache.add_change(a_change);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::{
            submessage_elements::{Parameter, ParameterListSubmessageElementType},
            types::SubmessageFlag,
        },
        structure::types::{EntityId, SequenceNumber, GUIDPREFIX_UNKNOWN, GUID_UNKNOWN},
    };

    use super::*;

    struct MockCacheChange {
        kind: ChangeKind,
        writer_guid: GUID,
        instance_handle: i32,
        sequence_number: SequenceNumber,
        data_value: [u8; 3],
        inline_qos: (),
    }

    impl<'a> RTPSCacheChangeOperations<'a> for MockCacheChange {
        type DataType = &'a [u8];
        type InstanceHandleType = i32;
        type InlineQosType = ();

        fn new(
            kind: ChangeKind,
            writer_guid: GUID,
            instance_handle: Self::InstanceHandleType,
            sequence_number: crate::structure::types::SequenceNumber,
            data: Self::DataType,
            inline_qos: Self::InlineQosType,
        ) -> Self {
            let mut data_value: [u8; 3] = [0; 3];
            data_value.clone_from_slice(data);
            Self {
                kind,
                writer_guid,
                instance_handle,
                sequence_number,
                data_value,
                inline_qos,
            }
        }
    }

    struct MockHistoryCache(Option<MockCacheChange>);

    impl RTPSHistoryCache for MockHistoryCache {
        type CacheChange = MockCacheChange;

        fn new() -> Self
        where
            Self: Sized,
        {
            todo!()
        }

        fn add_change(&mut self, change: Self::CacheChange) {
            self.0 = Some(change)
        }

        fn remove_change(&mut self, _seq_num: &crate::structure::types::SequenceNumber) {
            todo!()
        }

        fn get_change(
            &self,
            _seq_num: &crate::structure::types::SequenceNumber,
        ) -> Option<&Self::CacheChange> {
            todo!()
        }

        fn get_seq_num_min(&self) -> Option<crate::structure::types::SequenceNumber> {
            todo!()
        }

        fn get_seq_num_max(&self) -> Option<crate::structure::types::SequenceNumber> {
            todo!()
        }
    }

    struct MockStatelessReader {
        reader_cache: MockHistoryCache,
    }

    impl RTPSEntity for MockStatelessReader {
        fn guid(&self) -> &GUID {
            &GUID_UNKNOWN
        }
    }

    impl RTPSReader for MockStatelessReader {
        type HistoryCacheType = MockHistoryCache;

        fn heartbeat_response_delay(&self) -> &crate::behavior::types::Duration {
            todo!()
        }

        fn heartbeat_supression_duration(&self) -> &crate::behavior::types::Duration {
            todo!()
        }

        fn reader_cache(&self) -> &Self::HistoryCacheType {
            todo!()
        }

        fn reader_cache_mut(&mut self) -> &mut Self::HistoryCacheType {
            &mut self.reader_cache
        }

        fn expects_inline_qos(&self) -> bool {
            todo!()
        }
    }

    pub struct MockEntityIdSubmessageElement(EntityId);

    impl EntityIdSubmessageElementType for MockEntityIdSubmessageElement {
        fn new(_value: &crate::structure::types::EntityId) -> Self {
            todo!()
        }

        fn value(&self) -> crate::structure::types::EntityId {
            self.0
        }
    }
    pub struct MockSequenceNumberSubmessageElement(SequenceNumber);

    impl SequenceNumberSubmessageElementType for MockSequenceNumberSubmessageElement {
        fn new(_value: &crate::structure::types::SequenceNumber) -> Self {
            todo!()
        }

        fn value(&self) -> crate::structure::types::SequenceNumber {
            self.0
        }
    }

    pub struct MockParameterListSubmessageElement;

    impl<'a> ParameterListSubmessageElementType<'a> for MockParameterListSubmessageElement {
        type IntoIter = Option<Parameter<'a>>;

        fn new(_parameter: &[crate::messages::submessage_elements::Parameter]) -> Self {
            todo!()
        }

        fn parameter(&'a self) -> Self::IntoIter {
            todo!()
        }
    }

    pub struct MockSerializedDataSubmessageElement<'a>(&'a [u8]);

    impl<'a> SerializedDataSubmessageElementType<'a> for MockSerializedDataSubmessageElement<'a> {
        type Value = &'a [u8];

        fn new(_value: &Self::Value) -> Self {
            todo!()
        }

        fn value(&self) -> Self::Value {
            self.0
        }
    }

    pub struct MockDataSubmessage<'a> {
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        reader_id: MockEntityIdSubmessageElement,
        writer_id: MockEntityIdSubmessageElement,
        writer_sn: MockSequenceNumberSubmessageElement,
        serialized_payload: MockSerializedDataSubmessageElement<'a>,
    }

    impl<'a> DataSubmessage<'a> for MockDataSubmessage<'a> {
        type EntityIdSubmessageElementType = MockEntityIdSubmessageElement;

        type SequenceNumberSubmessageElementType = MockSequenceNumberSubmessageElement;

        type ParameterListSubmessageElementType = MockParameterListSubmessageElement;

        type SerializedDataSubmessageElementType = MockSerializedDataSubmessageElement<'a>;

        fn new(
            _endianness_flag: crate::messages::types::SubmessageFlag,
            _inline_qos_flag: crate::messages::types::SubmessageFlag,
            _data_flag: crate::messages::types::SubmessageFlag,
            _key_flag: crate::messages::types::SubmessageFlag,
            _non_standard_payload_flag: crate::messages::types::SubmessageFlag,
            _reader_id: Self::EntityIdSubmessageElementType,
            _writer_id: Self::EntityIdSubmessageElementType,
            _writer_sn: Self::SequenceNumberSubmessageElementType,
            _inline_qos: Self::ParameterListSubmessageElementType,
            _serialized_payload: Self::SerializedDataSubmessageElementType,
        ) -> Self {
            todo!()
        }

        fn endianness_flag(&self) -> crate::messages::types::SubmessageFlag {
            todo!()
        }

        fn inline_qos_flag(&self) -> crate::messages::types::SubmessageFlag {
            todo!()
        }

        fn data_flag(&self) -> crate::messages::types::SubmessageFlag {
            self.data_flag
        }

        fn key_flag(&self) -> crate::messages::types::SubmessageFlag {
            self.key_flag
        }

        fn non_standard_payload_flag(&self) -> crate::messages::types::SubmessageFlag {
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
            todo!()
        }

        fn serialized_payload(&self) -> &Self::SerializedDataSubmessageElementType {
            &self.serialized_payload
        }
    }

    #[test]
    fn receive_data_one_cache_change() {
        let mut stateless_reader = MockStatelessReader {
            reader_cache: MockHistoryCache(None),
        };
        let source_guid_prefix = GUIDPREFIX_UNKNOWN;
        let writer_entity_id = EntityId {
            entity_key: [1, 2, 3],
            entity_kind: crate::structure::types::EntityKind::BuiltInWriterWithKey,
        };
        let message_sequence_number = 1;
        let data = MockDataSubmessage {
            data_flag: true,
            key_flag: false,
            reader_id: MockEntityIdSubmessageElement(ENTITYID_UNKNOWN),
            writer_id: MockEntityIdSubmessageElement(writer_entity_id),
            writer_sn: MockSequenceNumberSubmessageElement(message_sequence_number),
            serialized_payload: MockSerializedDataSubmessageElement(&[1, 2, 3]),
        };
        stateless_reader.receive_data(source_guid_prefix, &data);

        if let Some(cache_change) = &stateless_reader.reader_cache.0 {
            assert_eq!(cache_change.kind, ChangeKind::Alive);
            assert_eq!(
                cache_change.writer_guid,
                GUID::new(source_guid_prefix, writer_entity_id)
            );
            assert_eq!(cache_change.sequence_number, message_sequence_number);
            assert_eq!(cache_change.data_value, [1, 2, 3]);
            assert_eq!(cache_change.inline_qos, ());
            assert_eq!(cache_change.instance_handle, 0);
        } else {
            panic!("Cache change not created")
        }
    }
}
