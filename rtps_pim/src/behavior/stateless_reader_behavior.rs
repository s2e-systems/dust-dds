use crate::{
    messages::submessages::DataSubmessage,
    structure::{history_cache::RtpsHistoryCacheOperations, types::GuidPrefix},
};

use super::{
    reader::reader::RtpsReaderAttributes,
    stateful_reader_behavior::TryFromDataSubmessageAndGuidPrefix,
};

pub trait RtpsStatelessReaderReceiveDataSubmessage<P, D> {
    fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessage<P, D>,
        source_guid_prefix: GuidPrefix,
    );
}

pub trait BestEffortStatelessReaderReceiveDataBehavior<P, D> {
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, D>);
}

impl<T, P, D> BestEffortStatelessReaderReceiveDataBehavior<P, D> for T
where
    T: RtpsReaderAttributes,
    T::HistoryCacheType: RtpsHistoryCacheOperations,
    <T::HistoryCacheType as RtpsHistoryCacheOperations>::CacheChangeType:
        TryFromDataSubmessageAndGuidPrefix<P, D>,
{
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, D>) {
        if let Ok(a_change) = TryFromDataSubmessageAndGuidPrefix::from(source_guid_prefix, data) {
            self.reader_cache().add_change(a_change);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        messages::submessage_elements::{
            EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
        },
        structure::types::{SequenceNumber, ENTITYID_UNKNOWN},
    };

    use super::*;

    use mockall::mock;

    // Cache change is not mocked with the mocking framework since
    // both the constructor and the attributes don't need to be defined as part of the test run
    #[derive(Debug, PartialEq)]
    struct MockCacheChange;

    impl<P, D> TryFromDataSubmessageAndGuidPrefix<P, D> for MockCacheChange {
        type Error = ();
        fn from(
            _source_guid_prefix: GuidPrefix,
            _data: &DataSubmessage<P, D>,
        ) -> Result<Self, Self::Error> {
            Ok(MockCacheChange)
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

    struct MockStatelessReader {
        reader_cache: MockHistoryCache,
    }

    impl RtpsReaderAttributes for MockStatelessReader {
        type HistoryCacheType = MockHistoryCache;

        fn heartbeat_response_delay(&self) -> crate::behavior::types::Duration {
            todo!()
        }

        fn heartbeat_suppression_duration(&self) -> crate::behavior::types::Duration {
            todo!()
        }

        fn reader_cache(&mut self) -> &mut Self::HistoryCacheType {
            &mut self.reader_cache
        }

        fn expects_inline_qos(&self) -> bool {
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
        let mut stateless_reader = MockStatelessReader { reader_cache };
        BestEffortStatelessReaderReceiveDataBehavior::receive_data(
            &mut stateless_reader,
            source_guid_prefix,
            &data,
        );
    }
}
