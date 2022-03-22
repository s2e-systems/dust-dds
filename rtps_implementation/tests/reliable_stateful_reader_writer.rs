use mockall::mock;
use rtps_implementation::{
    rtps_reader_proxy_impl::RtpsReaderProxyImpl,
    rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
    rtps_stateful_writer_impl::{RtpsStatefulSubmessage, RtpsStatefulWriterImpl},
    rtps_writer_proxy_impl::RtpsWriterProxyImpl,
    utils::clock::{Timer, TimerConstructor},
};
use rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderAttributes,
            stateful_reader::{RtpsStatefulReaderConstructor, RtpsStatefulReaderOperations},
            writer_proxy::{
                RtpsWriterProxyAttributes, RtpsWriterProxyConstructor, RtpsWriterProxyOperations,
            },
        },
        types::{Duration, DURATION_ZERO},
        writer::{
            reader_proxy::{RtpsReaderProxyAttributes, RtpsReaderProxyConstructor},
            stateful_writer::{RtpsStatefulWriterConstructor, RtpsStatefulWriterOperations},
            writer::{RtpsWriterAttributes, RtpsWriterOperations},
        },
    },
    messages::{submessages::DataSubmessage, types::Count},
    structure::{
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{
            ChangeKind, EntityId, Guid, GuidPrefix, ReliabilityKind, TopicKind, ENTITYID_UNKNOWN,
        },
    },
};

mock! {
    Timer {}

    impl Timer for Timer {
        fn reset(&mut self);
        fn elapsed(&self) -> std::time::Duration;
    }
}

impl TimerConstructor for MockTimer {
    fn new() -> Self {
        MockTimer::new()
    }
}

#[test]
fn reliable_stateful_reader_writer_dropped_data() {
    let writer_guid = Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1));
    let reader_guid = Guid::new(GuidPrefix([2; 12]), EntityId::new([2; 3], 2));

    let mut stateful_writer = RtpsStatefulWriterImpl::<MockTimer>::new(
        writer_guid,
        TopicKind::NoKey,
        ReliabilityKind::Reliable,
        &[],
        &[],
        true,
        Duration::new(1, 0),
        DURATION_ZERO,
        DURATION_ZERO,
        None,
    );

    let mut stateful_reader = RtpsStatefulReaderImpl::new(
        reader_guid,
        TopicKind::NoKey,
        ReliabilityKind::Reliable,
        &[],
        &[],
        Duration {
            seconds: 1,
            fraction: 0,
        },
        DURATION_ZERO,
        false,
    );

    let a_reader_proxy =
        RtpsReaderProxyImpl::new(reader_guid, ENTITYID_UNKNOWN, &[], &[], false, true);
    stateful_writer.matched_reader_add(a_reader_proxy);

    let a_writer_proxy = RtpsWriterProxyImpl::new(writer_guid, &[], &[], None, ENTITYID_UNKNOWN);
    stateful_reader.matched_writer_add(a_writer_proxy);

    // Send and receive first heartbeat (no data sent)
    {
        stateful_writer
            .heartbeat_timer
            .expect_elapsed()
            .return_const(std::time::Duration::from_secs(0));

        // no heartbeat before delay
        assert_eq!(0, stateful_writer.produce_destined_submessages().len());

        stateful_writer.heartbeat_timer.checkpoint();
        stateful_writer
            .heartbeat_timer
            .expect_elapsed()
            .return_const(std::time::Duration::from_secs(1));
        stateful_writer
            .heartbeat_timer
            .expect_reset()
            .once()
            .return_const(());

        // one heartbeat to send
        {
            let destined_submessages = stateful_writer.produce_destined_submessages();
            assert_eq!(1, destined_submessages.len());

            let (reader_proxy, heartbeats) = &destined_submessages[0];
            assert_eq!(reader_guid, reader_proxy.remote_reader_guid());
            assert_eq!(1, heartbeats.len());

            if let RtpsStatefulSubmessage::Heartbeat(heartbeat) = &heartbeats[0] {
                assert_eq!(1, heartbeat.first_sn.value);
                assert_eq!(0, heartbeat.last_sn.value);
                assert_eq!(Count(1), heartbeat.count.value);

                stateful_reader.process_heartbeat_submessage(heartbeat, writer_guid.prefix);
            } else {
                panic!("Not a heartbeat (;_;)");
            }

            assert!(stateful_reader.matched_writers[0]
                .missing_changes()
                .is_empty());
            assert!(stateful_reader.produce_acknack_submessages().is_empty());
        }

        stateful_writer.heartbeat_timer.checkpoint();
        stateful_writer
            .heartbeat_timer
            .expect_elapsed()
            .return_const(std::time::Duration::from_secs(0));
    }

    // Write 5 changes
    {
        let changes = vec![
            stateful_writer.new_change(ChangeKind::Alive, vec![0, 1], vec![], 0), // SN: 1
            stateful_writer.new_change(ChangeKind::Alive, vec![2, 3], vec![], 0), // SN: 2
            stateful_writer.new_change(ChangeKind::Alive, vec![4, 5], vec![], 0), // SN: 3
            stateful_writer.new_change(ChangeKind::Alive, vec![6, 7], vec![], 0), // SN: 4
            stateful_writer.new_change(ChangeKind::Alive, vec![8, 9], vec![], 0), // SN: 5
        ];

        for change in changes {
            stateful_writer.writer_cache().add_change(change);
        }
    }

    // Receive only messages 2 and 4
    {
        let mut destined_submessages = stateful_writer.produce_destined_submessages();
        assert_eq!(1, destined_submessages.len());

        let (reader_proxy, submessages) = &mut destined_submessages[0];
        assert_eq!(reader_guid, reader_proxy.remote_reader_guid());
        assert_eq!(5, submessages.len());

        // drop messages 1, 3 and 5
        submessages.retain(|message| {
            matches!(
                message,
                RtpsStatefulSubmessage::Data(DataSubmessage{writer_sn, ..})
                if writer_sn.value % 2 == 0
            )
        });
        assert_eq!(2, submessages.len()); // 2 and 4

        for submessage in submessages {
            match submessage {
                RtpsStatefulSubmessage::Data(data) => {
                    stateful_reader.process_data_submessage(data, writer_guid.prefix)
                }
                _ => panic!("This is not data (;_;)"),
            }
        }

        assert_eq!(2, stateful_reader.reader_cache().changes().len()); // has received 2 and 4
        assert_eq!(
            vec![1, 3],
            stateful_reader.matched_writers[0].missing_changes()
        ); // knows at least 1 and 3 are missing
    }

    // No AckNack sent before receiving the heartbeat
    assert!(stateful_reader.produce_acknack_submessages().is_empty());

    // Send and receive second heartbeat
    {
        stateful_writer.heartbeat_timer.checkpoint();
        stateful_writer
            .heartbeat_timer
            .expect_elapsed()
            .return_const(std::time::Duration::from_secs(1));
        stateful_writer
            .heartbeat_timer
            .expect_reset()
            .once()
            .return_const(());

        // one heartbeat to send
        {
            let destined_submessages = stateful_writer.produce_destined_submessages();
            assert_eq!(1, destined_submessages.len());

            let (reader_proxy, heartbeats) = &destined_submessages[0];
            assert_eq!(reader_guid, reader_proxy.remote_reader_guid());
            assert_eq!(1, heartbeats.len());

            if let RtpsStatefulSubmessage::Heartbeat(heartbeat) = &heartbeats[0] {
                assert_eq!(1, heartbeat.first_sn.value);
                assert_eq!(5, heartbeat.last_sn.value);
                assert_eq!(Count(2), heartbeat.count.value);

                stateful_reader.process_heartbeat_submessage(heartbeat, writer_guid.prefix);
            } else {
                panic!("Not a heartbeat (;_;)");
            }

            assert_eq!(
                vec![1, 3, 5],
                stateful_reader.matched_writers[0].missing_changes()
            );
        }

        stateful_writer.heartbeat_timer.checkpoint();
        stateful_writer
            .heartbeat_timer
            .expect_elapsed()
            .return_const(std::time::Duration::from_secs(0));
    }

    // Send and receive AckNack
    {
        let messages = stateful_reader.produce_acknack_submessages();
        assert_eq!(1, messages.len());
        let (writer_proxy, acknacks) = &messages[0];
        assert_eq!(writer_guid, writer_proxy.remote_writer_guid());
        assert_eq!(1, acknacks.len());

        assert_eq!(vec![1, 3, 5], acknacks[0].reader_sn_state.set);

        stateful_writer.process_acknack_submessage(&acknacks[0], reader_guid.prefix);
    }

    // Do not resend the acknack
    assert!(stateful_reader.produce_acknack_submessages().is_empty());

    // Re-send missing messages
    {
        let destined_submessages = stateful_writer.produce_destined_submessages();
        assert_eq!(1, destined_submessages.len());
        let (reader_proxy, submessages) = &destined_submessages[0];
        assert_eq!(reader_guid, reader_proxy.remote_reader_guid());
        assert_eq!(3, submessages.len());

        for submessage in submessages {
            match submessage {
                RtpsStatefulSubmessage::Data(data) => {
                    stateful_reader.process_data_submessage(data, writer_guid.prefix)
                }
                _ => panic!("This is not data (;_;)"),
            }
        }

        assert_eq!(5, stateful_reader.reader_cache().changes().len());
        assert!(stateful_reader.matched_writers[0]
            .missing_changes()
            .is_empty());
    }

    // Verify that all messages were received
    let mut data: Vec<u8> = stateful_reader
        .reader_cache()
        .changes()
        .iter()
        .flat_map(|change| change.data.0.iter().cloned())
        .collect();
    data.sort();
    assert_eq!(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], data);

    // Send and receive third heartbeat (no new data)
    {
        stateful_writer.heartbeat_timer.checkpoint();
        stateful_writer
            .heartbeat_timer
            .expect_elapsed()
            .return_const(std::time::Duration::from_secs(1));
        stateful_writer
            .heartbeat_timer
            .expect_reset()
            .once()
            .return_const(());

        // one heartbeat to send
        {
            let destined_submessages = stateful_writer.produce_destined_submessages();
            assert_eq!(1, destined_submessages.len());

            let (reader_proxy, heartbeats) = &destined_submessages[0];
            assert_eq!(reader_guid, reader_proxy.remote_reader_guid());
            assert_eq!(1, heartbeats.len());

            if let RtpsStatefulSubmessage::Heartbeat(heartbeat) = &heartbeats[0] {
                assert_eq!(1, heartbeat.first_sn.value);
                assert_eq!(5, heartbeat.last_sn.value);
                assert_eq!(Count(3), heartbeat.count.value);

                stateful_reader.process_heartbeat_submessage(heartbeat, writer_guid.prefix);
            } else {
                panic!("Not a heartbeat (;_;)");
            }

            assert!(stateful_reader.matched_writers[0]
                .missing_changes()
                .is_empty());
            assert!(stateful_reader.produce_acknack_submessages().is_empty());
        }
    }
}
