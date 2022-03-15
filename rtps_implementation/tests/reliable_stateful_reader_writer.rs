use rtps_implementation::{
    rtps_reader_proxy_impl::RtpsReaderProxyImpl,
    rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
    rtps_stateful_writer_impl::{RtpsStatefulSubmessage, RtpsStatefulWriterImpl},
    rtps_writer_proxy_impl::RtpsWriterProxyImpl,
};
use rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderAttributes,
            stateful_reader::{RtpsStatefulReaderConstructor, RtpsStatefulReaderOperations},
            writer_proxy::RtpsWriterProxyConstructor,
        },
        types::{Duration, DURATION_ZERO},
        writer::{
            reader_proxy::RtpsReaderProxyConstructor,
            stateful_writer::{RtpsStatefulWriterConstructor, RtpsStatefulWriterOperations},
            writer::{RtpsWriterAttributes, RtpsWriterOperations},
        },
    },
    structure::{
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{
            ChangeKind, EntityId, Guid, GuidPrefix, ReliabilityKind, TopicKind, ENTITYID_UNKNOWN,
        },
    },
};

#[test]
fn reliable_stateful_reader_writer_dropped_data() {
    let writer_guid = Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1));
    let writer_topic_kind = TopicKind::NoKey;
    let writer_reliability_level = ReliabilityKind::Reliable;
    let writer_unicast_locator_list = &[];
    let writer_multicast_locator_list = &[];
    let writer_push_mode = true;
    let writer_heartbeat_period = Duration {
        seconds: 1,
        fraction: 0,
    };
    let writer_nack_response_delay = DURATION_ZERO;
    let writer_nack_suppression_duration = DURATION_ZERO;
    let writer_data_max_size_serialized = None;
    let mut stateful_writer = RtpsStatefulWriterImpl::new(
        writer_guid,
        writer_topic_kind,
        writer_reliability_level,
        writer_unicast_locator_list,
        writer_multicast_locator_list,
        writer_push_mode,
        writer_heartbeat_period,
        writer_nack_response_delay,
        writer_nack_suppression_duration,
        writer_data_max_size_serialized,
    );

    let reader_guid = Guid::new(GuidPrefix([2; 12]), EntityId::new([2; 3], 2));
    let reader_topic_kind = TopicKind::NoKey;
    let reader_reliability_level = ReliabilityKind::Reliable;
    let reader_unicast_locator_list = &[];
    let reader_multicast_locator_list = &[];
    let reader_heartbeat_response_delay = DURATION_ZERO;
    let reader_heartbeat_suppression_duration = DURATION_ZERO;
    let reader_expects_inline_qos = false;
    let mut stateful_reader = RtpsStatefulReaderImpl::new(
        reader_guid,
        reader_topic_kind,
        reader_reliability_level,
        reader_unicast_locator_list,
        reader_multicast_locator_list,
        reader_heartbeat_response_delay,
        reader_heartbeat_suppression_duration,
        reader_expects_inline_qos,
    );

    let a_reader_proxy = RtpsReaderProxyImpl::new(
        reader_guid,
        ENTITYID_UNKNOWN,
        reader_unicast_locator_list,
        reader_multicast_locator_list,
        reader_expects_inline_qos,
        true,
    );
    stateful_writer.matched_reader_add(a_reader_proxy);

    let a_writer_proxy = RtpsWriterProxyImpl::new(
        writer_guid,
        writer_unicast_locator_list,
        writer_multicast_locator_list,
        writer_data_max_size_serialized,
        ENTITYID_UNKNOWN,
    );
    stateful_reader.matched_writer_add(a_writer_proxy);

    let change1 = stateful_writer.new_change(ChangeKind::Alive, vec![1, 2, 3, 4], vec![], 0);
    let change2 = stateful_writer.new_change(ChangeKind::Alive, vec![5, 6, 7, 8], vec![], 0);
    stateful_writer.writer_cache().add_change(change1);
    stateful_writer.writer_cache().add_change(change2);

    let mut writer_destined_submessages = stateful_writer.produce_destined_submessages();
    let (_reader_proxy, writer_submessages) = writer_destined_submessages.pop().unwrap();

    for submessage in writer_submessages {
        match submessage {
            RtpsStatefulSubmessage::Data(d) => {
                stateful_reader.process_data_submessage(&d, writer_guid.prefix())
            }
            RtpsStatefulSubmessage::Gap(g) => {
                stateful_reader.process_gap_submessage(&g, writer_guid.prefix())
            }
        }
    }

    assert_eq!(2, stateful_reader.reader_cache().changes().len())
}
