use crate::{
    messages::submessages::DataSubmessage,
    structure::{
        cache_change::RtpsCacheChangeConstructor,
        history_cache::RtpsHistoryCacheOperations,
        types::{ChangeKind, Guid, GuidPrefix},
    },
};

pub trait BestEffortStatelessReaderReceiveDataBehavior<P, D> {
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, D>);
}

impl<T, P, D, C> BestEffortStatelessReaderReceiveDataBehavior<P, D> for T
where
    T: RtpsHistoryCacheOperations<CacheChangeType = C>,
    C: RtpsCacheChangeConstructor,
    for<'b> &'b D: Into<<C as RtpsCacheChangeConstructor>::DataType>,
    for<'b> &'b P: Into<<C as RtpsCacheChangeConstructor>::ParameterListType>,
{
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, D>) {
        let kind = match (data.data_flag, data.key_flag) {
            (true, false) => ChangeKind::Alive,
            (false, true) => ChangeKind::NotAliveDisposed,
            _ => todo!(),
        };
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value);
        let instance_handle = 0;
        let sequence_number = data.writer_sn.value;
        let data_value = (&data.serialized_payload.value).into();
        let inline_qos = (&data.inline_qos.parameter).into();
        let a_change = C::new(
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data_value,
            inline_qos,
        );
        self.add_change(a_change);
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        messages::submessage_elements::{
            EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
        },
        structure::types::{InstanceHandle, SequenceNumber, ENTITYID_UNKNOWN},
    };

    use super::*;

    use mockall::mock;

    struct MockData;
    impl<T> From<&T> for MockData {
        fn from(_: &T) -> Self {
            MockData
        }
    }

    struct MockParameterList;
    impl From<&()> for MockParameterList {
        fn from(_: &()) -> Self {
            MockParameterList
        }
    }

    // Cache change is not mocked with the mocking framework since
    // both the constructor and the attributes don't need to be defined as part of the test run
    #[derive(Debug, PartialEq)]
    struct MockCacheChange;

    impl RtpsCacheChangeConstructor for MockCacheChange {
        type DataType = MockData;
        type ParameterListType = MockParameterList;

        fn new(
            _kind: ChangeKind,
            _writer_guid: Guid,
            _instance_handle: InstanceHandle,
            _sequence_number: SequenceNumber,
            _data_value: Self::DataType,
            _inline_qos: Self::ParameterListType,
        ) -> Self {
            Self
        }
    }

    mock! {
        HistoryCache{
            fn add_change_(&mut self, change: MockCacheChange);
        }
    }

    impl RtpsHistoryCacheOperations for MockHistoryCache {
        type CacheChangeType = MockCacheChange;

        fn add_change(&mut self, change: Self::CacheChangeType) {
            self.add_change_(change)
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

    #[test]
    fn best_effort_stateless_reader_receive_data() {
        let mut reader_cache = MockHistoryCache::new();
        let source_guid_prefix = GuidPrefix([1; 12]);
        let data = DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement { parameter: () },
            serialized_payload: SerializedDataSubmessageElement {
                value: &[1_u8, 2, 3, 4][..],
            },
        };
        reader_cache.expect_add_change_().once().return_const(());

        BestEffortStatelessReaderReceiveDataBehavior::receive_data(
            &mut reader_cache,
            source_guid_prefix,
            &data,
        );
    }
}
