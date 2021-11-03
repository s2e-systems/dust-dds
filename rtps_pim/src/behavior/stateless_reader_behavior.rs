use crate::{
    messages::{submessage_elements::Parameter, submessages::DataSubmessage},
    structure::{
        cache_change::RtpsCacheChange,
        history_cache::RtpsHistoryCacheAddChange,
        types::{ChangeKind, Guid, GuidPrefix, ENTITYID_UNKNOWN},
    },
};

use super::reader::reader::RtpsReader;

pub trait StatelessReaderBehavior<P> {
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, &[u8]>);
}

impl<'a, 'b, L, C, P> StatelessReaderBehavior<P> for RtpsReader<L, C>
where
    C: for<'c> RtpsHistoryCacheAddChange<&'c [Parameter<&'c [u8]>], &'c [u8]>,
    P: AsRef<[Parameter<&'a [u8]>]>,
{
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, &[u8]>) {
        let reader_id = data.reader_id.value;
        if &reader_id == self.guid.entity_id() || reader_id == ENTITYID_UNKNOWN {
            let kind = match (data.data_flag, data.key_flag) {
                (true, false) => ChangeKind::Alive,
                (false, true) => ChangeKind::NotAliveDisposed,
                _ => todo!(),
            };
            let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value);
            let instance_handle = 0;
            let sequence_number = data.writer_sn.value;
            let data_value = data.serialized_payload.value;
            let inline_qos = data.inline_qos.parameter.as_ref();
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
        behavior::{reader::reader::RtpsReader, types::DURATION_ZERO},
        messages::submessage_elements::{
            EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
        },
        structure::{
            history_cache::RtpsHistoryCacheConstructor,
            types::{
                EntityId, InstanceHandle, ReliabilityKind, SequenceNumber, TopicKind,
                BUILT_IN_WRITER_WITH_KEY, GUIDPREFIX_UNKNOWN,
            },
        },
    };

    use super::*;

    struct MockCacheChange {
        kind: ChangeKind,
        writer_guid: Guid,
        sequence_number: SequenceNumber,
        instance_handle: InstanceHandle,
        data: [u8; 1],
        inline_qos: (),
    }

    struct MockHistoryCache(Option<MockCacheChange>);

    impl RtpsHistoryCacheConstructor for MockHistoryCache {
        fn new() -> Self {
            MockHistoryCache(None)
        }
    }

    impl RtpsHistoryCacheAddChange<&'_ [Parameter<&'_ [u8]>], &'_ [u8]> for MockHistoryCache {
        fn add_change(&mut self, change: RtpsCacheChange<&'_ [Parameter<&'_ [u8]>], &'_ [u8]>) {
            self.0 = Some(MockCacheChange {
                kind: change.kind,
                writer_guid: change.writer_guid,
                sequence_number: change.sequence_number,
                instance_handle: change.instance_handle,
                data: [change.data_value[0].clone()],
                inline_qos: (),
            });
        }
    }

    #[test]
    fn receive_data_one_cache_change() {
        let mut stateless_reader: RtpsReader<(), MockHistoryCache> = RtpsReader::new(
            Guid {
                prefix: GuidPrefix([1; 12]),
                entity_id: EntityId::new([0; 3], 1),
            },
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            (),
            (),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        );

        let source_guid_prefix = GUIDPREFIX_UNKNOWN;
        let writer_entity_id = EntityId::new([1, 2, 3], BUILT_IN_WRITER_WITH_KEY);
        let message_sequence_number = 1;
        let data = DataSubmessage {
            endianness_flag: false,
            inline_qos_flag: false,
            non_standard_payload_flag: false,
            data_flag: true,
            key_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: writer_entity_id,
            },
            writer_sn: SequenceNumberSubmessageElement {
                value: message_sequence_number,
            },
            serialized_payload: SerializedDataSubmessageElement { value: &[3][..] },
            inline_qos: ParameterListSubmessageElement { parameter: [] },
        };
        stateless_reader.receive_data(source_guid_prefix, &data);

        if let Some(cache_change) = &stateless_reader.reader_cache.0 {
            assert_eq!(cache_change.kind, ChangeKind::Alive);
            assert_eq!(
                cache_change.writer_guid,
                Guid::new(source_guid_prefix, writer_entity_id)
            );
            assert_eq!(cache_change.sequence_number, message_sequence_number);
            assert_eq!(cache_change.data, [3]);
            assert_eq!(cache_change.inline_qos, ());
            assert_eq!(cache_change.instance_handle, 0);
        } else {
            panic!("Cache change not created")
        }
    }
}
