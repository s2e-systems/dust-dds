use std::cell::RefCell;

use mockall::mock;
use rtps_implementation::{
    rtps_reader_proxy_impl::RtpsReaderProxyImpl,
    rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
    rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
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
        stateful_writer_behavior::{
            ReliableReaderProxyReceiveAcknackBehavior, ReliableReaderProxyRequestedChangesBehavior,
            StatefulWriterSendSubmessages,
        },
        types::{Duration, DURATION_ZERO},
        writer::{
            reader_proxy::{RtpsReaderProxyAttributes, RtpsReaderProxyConstructor},
            stateful_writer::{
                RtpsStatefulWriterAttributes, RtpsStatefulWriterConstructor,
                RtpsStatefulWriterOperations,
            },
            writer::RtpsWriterOperations,
        },
    },
    messages::{
        submessage_elements::Parameter,
        submessages::{DataSubmessage, GapSubmessage, HeartbeatSubmessage}, types::Count,
    },
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

struct DestinedSubmessages<'a> {
    data: Vec<DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>>,
    gaps: Vec<GapSubmessage<Vec<i64>>>,
    heartbeats: Vec<HeartbeatSubmessage>,
}

impl<'a> DestinedSubmessages<'a> {
    fn len(&self) -> usize {
        self.data.len() + self.gaps.len() + self.heartbeats.len()
    }
}

fn produce_destined_submessages<'a, T: Timer>(
    stateful_writer: &'a mut RtpsStatefulWriterImpl<T>,
) -> DestinedSubmessages<'a> {
    let data = RefCell::new(Vec::new());
    let gaps = RefCell::new(Vec::new());
    let heartbeats = RefCell::new(Vec::new());

    StatefulWriterSendSubmessages::send_submessages(
        stateful_writer,
        |_, datum| data.borrow_mut().push(datum),
        |_, gap| gaps.borrow_mut().push(gap),
        |_, heartbeat| heartbeats.borrow_mut().push(heartbeat),
    );

    DestinedSubmessages {
        data: data.take(),
        gaps: gaps.take(),
        heartbeats: heartbeats.take(),
    }
}

struct RequestedChanges<'a> {
    data: Vec<DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>>,
    gaps: Vec<GapSubmessage<Vec<i64>>>,
}

impl<'a> RequestedChanges<'a> {
    fn len(&self) -> usize {
        self.data.len() + self.gaps.len()
    }
}

fn produce_requested_submessages<'a, T: Timer>(
    stateful_writer: &'a mut RtpsStatefulWriterImpl<T>,
) -> RequestedChanges<'a> {
    let data = RefCell::new(Vec::new());
    let gaps = RefCell::new(Vec::new());

    for mut reader_proxy in stateful_writer.matched_readers() {
        reader_proxy.send_requested_changes(
            |_, datum| data.borrow_mut().push(datum),
            |_, gap| gaps.borrow_mut().push(gap),
        );
    }

    RequestedChanges {
        data: data.take(),
        gaps: gaps.take(),
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
        StatefulWriterSendSubmessages::send_submessages(
            &mut stateful_writer,
            |_, _| panic!("No data should be sent"),
            |_, _| panic!("No gap should be sent"),
            |_, _| panic!("No heartbeat should be sent"),
        );

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
            let destined_submessages = produce_destined_submessages(&mut stateful_writer);
            assert_eq!(1, destined_submessages.len());
            assert_eq!(1, destined_submessages.heartbeats.len());

            let heartbeat = &destined_submessages.heartbeats[0];
            assert_eq!(1, heartbeat.first_sn.value);
            assert_eq!(0, heartbeat.last_sn.value);
            assert_eq!(Count(1), heartbeat.count.value);

            stateful_reader.process_heartbeat_submessage(&heartbeat, writer_guid.prefix);

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
            stateful_writer.add_change(change);
        }
    }

    // Receive only messages 2 and 4
    {
        let mut destined_submessages = produce_destined_submessages(&mut stateful_writer);
        assert_eq!(5, destined_submessages.len());
        assert_eq!(5, destined_submessages.data.len());

        // drop messages 1, 3 and 5
        destined_submessages
            .data
            .retain(|message| message.writer_sn.value % 2 == 0);
        assert_eq!(2, destined_submessages.data.len()); // 2 and 4

        for data_submessage in destined_submessages.data {
            stateful_reader.process_data_submessage(&data_submessage, writer_guid.prefix)
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
            let destined_submessages = produce_destined_submessages(&mut stateful_writer);
            assert_eq!(1, destined_submessages.len());
            assert_eq!(1, destined_submessages.heartbeats.len());

            let heartbeat = &destined_submessages.heartbeats[0];
            assert_eq!(1, heartbeat.first_sn.value);
            assert_eq!(5, heartbeat.last_sn.value);
            assert_eq!(Count(2), heartbeat.count.value);

            stateful_reader.process_heartbeat_submessage(&heartbeat, writer_guid.prefix);
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
        assert_eq!(Count(1), acknacks[0].count.value);

        for mut reader_proxy in stateful_writer.matched_readers() {
            if reader_proxy.remote_reader_guid().prefix == reader_guid.prefix {
                reader_proxy.receive_acknack(&acknacks[0]);
            }
        }
    }

    // Do not resend the acknack
    assert!(stateful_reader.produce_acknack_submessages().is_empty());

    // Re-send missing messages
    {
        let requested_changes = produce_requested_submessages(&mut stateful_writer);
        assert_eq!(3, requested_changes.len());
        assert_eq!(3, requested_changes.data.len());

        for data_submessage in requested_changes.data {
            stateful_reader.process_data_submessage(&data_submessage, writer_guid.prefix)
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
        .flat_map(|change| change.data.iter().cloned())
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
            let destined_submessages = produce_destined_submessages(&mut stateful_writer);
            assert_eq!(1, destined_submessages.len());
            assert_eq!(1, destined_submessages.heartbeats.len());

            let heartbeat = &destined_submessages.heartbeats[0];
            assert_eq!(1, heartbeat.first_sn.value);
            assert_eq!(5, heartbeat.last_sn.value);
            assert_eq!(Count(3), heartbeat.count.value);

            stateful_reader.process_heartbeat_submessage(&heartbeat, writer_guid.prefix);

            assert!(stateful_reader.matched_writers[0]
                .missing_changes()
                .is_empty());
            assert!(stateful_reader.produce_acknack_submessages().is_empty());
        }
    }
}
