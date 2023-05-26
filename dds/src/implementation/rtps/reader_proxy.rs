use crate::infrastructure::time::Duration;

use super::{
    history_cache::{RtpsWriterCacheChange, WriterHistoryCache},
    messages::{
        submessages::{
            AckNackSubmessageRead, HeartbeatFragSubmessageWrite, HeartbeatSubmessageWrite,
            NackFragSubmessageRead,
        },
        types::FragmentNumber,
        RtpsSubmessageWriteKind,
    },
    types::{
        Count, DurabilityKind, EntityId, ExpectsInlineQos, Guid, Locator, ReliabilityKind,
        SequenceNumber,
    },
    utils::clock::{StdTimer, Timer, TimerConstructor},
    writer::RtpsWriter,
};

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatMachine {
    count: Count,
    reader_id: EntityId,
    timer: StdTimer,
}
impl HeartbeatMachine {
    fn new(reader_id: EntityId) -> Self {
        HeartbeatMachine {
            count: Count::new(0),
            reader_id,
            timer: StdTimer::new(),
        }
    }
    pub fn is_time_for_heartbeat(&self, heartbeat_period: Duration) -> bool {
        self.timer.elapsed()
            >= std::time::Duration::from_secs(heartbeat_period.sec() as u64)
                + std::time::Duration::from_nanos(heartbeat_period.nanosec() as u64)
    }
    pub fn submessage<'a>(
        &mut self,
        writer_id: EntityId,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
    ) -> RtpsSubmessageWriteKind<'a> {
        self.count = self.count.wrapping_add(1);
        self.timer.reset();
        RtpsSubmessageWriteKind::Heartbeat(HeartbeatSubmessageWrite {
            endianness_flag: true,
            final_flag: false,
            liveliness_flag: false,
            reader_id: self.reader_id,
            writer_id,
            first_sn,
            last_sn,
            count: self.count,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatFragMachine {
    count: Count,
    reader_id: EntityId,
}

impl HeartbeatFragMachine {
    fn new(reader_id: EntityId) -> Self {
        HeartbeatFragMachine {
            count: Count::new(0),
            reader_id,
        }
    }
    pub fn submessage<'a>(
        &mut self,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        last_fragment_num: FragmentNumber,
    ) -> RtpsSubmessageWriteKind<'a> {
        self.count = self.count.wrapping_add(1);
        RtpsSubmessageWriteKind::HeartbeatFrag(HeartbeatFragSubmessageWrite {
            endianness_flag: true,
            reader_id: self.reader_id,
            writer_id,
            writer_sn,
            last_fragment_num,
            count: self.count,
        })
    }
}

/// ChangeForReaderStatusKind
/// Enumeration used to indicate the status of a ChangeForReader. It can take the values:
/// UNSENT, UNACKNOWLEDGED, REQUESTED, ACKNOWLEDGED, UNDERWAY
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChangeForReaderStatusKind {
    Unsent,
    Unacknowledged,
    Requested,
    Acknowledged,
    Underway,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsReaderProxy {
    remote_reader_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    changes_for_reader: Vec<RtpsChangeForReader>,
    expects_inline_qos: ExpectsInlineQos,
    is_active: bool,
    last_received_acknack_count: Count,
    last_received_nack_frag_count: Count,
    heartbeat_machine: HeartbeatMachine,
    heartbeat_frag_machine: HeartbeatFragMachine,
    reliability: ReliabilityKind,
    durability: DurabilityKind,
}

impl RtpsReaderProxy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        remote_reader_guid: Guid,
        remote_group_entity_id: EntityId,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        expects_inline_qos: bool,
        is_active: bool,
        reliability: ReliabilityKind,
        durability: DurabilityKind,
    ) -> Self {
        let heartbeat_machine = HeartbeatMachine::new(remote_reader_guid.entity_id());
        let heartbeat_frag_machine = HeartbeatFragMachine::new(remote_reader_guid.entity_id());
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            changes_for_reader: vec![],
            expects_inline_qos: expects_inline_qos.into(),
            is_active,
            last_received_acknack_count: Count::new(0),
            last_received_nack_frag_count: Count::new(0),
            heartbeat_machine,
            heartbeat_frag_machine,
            reliability,
            durability,
        }
    }

    pub fn remote_reader_guid(&self) -> Guid {
        self.remote_reader_guid
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_slice()
    }

    pub fn changes_for_reader(&self) -> &[RtpsChangeForReader] {
        self.changes_for_reader.as_slice()
    }

    pub fn changes_for_reader_mut(&mut self) -> &mut Vec<RtpsChangeForReader> {
        &mut self.changes_for_reader
    }

    pub fn durability(&self) -> DurabilityKind {
        self.durability
    }

    pub fn receive_acknack(&mut self, acknack_submessage: &AckNackSubmessageRead) {
        match self.reliability {
            ReliabilityKind::BestEffort => (),
            ReliabilityKind::Reliable => {
                if acknack_submessage.count() > self.last_received_acknack_count {
                    self.acked_changes_set(acknack_submessage.reader_sn_state().base - 1);
                    self.requested_changes_set(acknack_submessage.reader_sn_state().set.as_ref());

                    self.last_received_acknack_count = acknack_submessage.count();
                }
            }
        }
    }

    pub fn receive_nack_frag(&mut self, nack_frag_submessage: &NackFragSubmessageRead) {
        match self.reliability {
            ReliabilityKind::BestEffort => (),
            ReliabilityKind::Reliable => {
                if nack_frag_submessage.count() > self.last_received_nack_frag_count {
                    self.requested_changes_set(&[nack_frag_submessage.writer_sn()]);
                    self.last_received_nack_frag_count = nack_frag_submessage.count();
                }
            }
        }
    }

    pub fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        // "FOR_EACH change in this.changes_for_reader
        // SUCH-THAT (change.sequenceNumber <= committed_seq_num) DO
        // change.status := ACKNOWLEDGED;"
        for change in &mut self.changes_for_reader {
            if change.sequence_number <= committed_seq_num {
                change.status = ChangeForReaderStatusKind::Acknowledged;
            }
        }
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        // "FOR_EACH seq_num IN req_seq_num_set DO
        //     FIND change_for_reader IN this.changes_for_reader
        //          SUCH-THAT (change_for_reader.sequenceNumber==seq_num)
        //     change_for_reader.status := REQUESTED;
        // END"
        for &seq_num in req_seq_num_set {
            for change_for_reader in &mut self
                .changes_for_reader
                .iter_mut()
                .filter(|change_for_reader| change_for_reader.sequence_number == seq_num)
            {
                change_for_reader.status = ChangeForReaderStatusKind::Requested;
            }
        }
    }
}

pub struct WriterAssociatedReaderProxy<'a> {
    writer: &'a RtpsWriter,
    reader_proxy: &'a mut RtpsReaderProxy,
}

impl<'a> WriterAssociatedReaderProxy<'a> {
    pub fn new(writer: &'a RtpsWriter, reader_proxy: &'a mut RtpsReaderProxy) -> Self {
        Self {
            writer,
            reader_proxy,
        }
    }

    pub fn remote_reader_guid(&self) -> Guid {
        self.reader_proxy.remote_reader_guid()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.reader_proxy.unicast_locator_list()
    }

    pub fn reliability(&self) -> ReliabilityKind {
        self.reader_proxy.reliability
    }

    pub fn heartbeat_machine(&mut self) -> &mut HeartbeatMachine {
        &mut self.reader_proxy.heartbeat_machine
    }

    pub fn heartbeat_frag_machine(&mut self) -> &mut HeartbeatFragMachine {
        &mut self.reader_proxy.heartbeat_frag_machine
    }

    // //////////////   ReaderProxy operations defined in the Rtps Standard

    pub fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        // "FOR_EACH change in this.changes_for_reader
        // SUCH-THAT (change.sequenceNumber <= committed_seq_num) DO
        // change.status := ACKNOWLEDGED;"
        for change in self.reader_proxy.changes_for_reader_mut() {
            if change.sequence_number <= committed_seq_num {
                change.status = ChangeForReaderStatusKind::Acknowledged;
            }
        }
    }

    pub fn next_requested_change(&mut self) -> RtpsChangeForReaderCacheChange<'a> {
        // "next_seq_num := MIN {change.sequenceNumber
        //     SUCH-THAT change IN this.requested_changes()}
        //  return change IN this.requested_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"
        let next_seq_num = self.requested_changes().iter().min().cloned().unwrap();

        let change = self
            .reader_proxy
            .changes_for_reader_mut()
            .iter_mut()
            .find(|c| c.sequence_number == next_seq_num)
            .unwrap();

        // Following 8.4.9.2.12 Transition T12 of Reliable Stateful Writer Behavior:
        // a_change := the_reader_proxy.next_requested_change();
        // a_change.status := UNDERWAY;
        // Note this is the only usage in the standard of next_requested_change() as such
        // the modification of the status is done always.
        change.status = ChangeForReaderStatusKind::Underway;

        // After ackNackSuppressionDuration = 0
        change.status = ChangeForReaderStatusKind::Unacknowledged;

        RtpsChangeForReaderCacheChange::new(change.clone(), self.writer.writer_cache())
    }

    pub fn next_unsent_change(&mut self) -> RtpsChangeForReaderCacheChange<'a> {
        // "next_seq_num := MIN { change.sequenceNumber
        //     SUCH-THAT change IN this.unsent_changes() };
        // return change IN this.unsent_changes()
        //     SUCH-THAT (change.sequenceNumber == next_seq_num);"
        let next_seq_num = self.unsent_changes().iter().min().cloned().unwrap();

        let change = self
            .reader_proxy
            .changes_for_reader_mut()
            .iter_mut()
            .find(|c| c.sequence_number() == next_seq_num)
            .unwrap();

        // Following 8.4.9.1.4 Transition T14 of BestEffort Stateful Writer Behavior:
        // a_change := the_reader_proxy.next_unsent_change();
        // a_change.status := UNDERWAY;
        // Note this is the only usage in the standard of next_unsent_change() as such
        // the modification of the status is done always.
        change.set_status(ChangeForReaderStatusKind::Underway);

        // After ackNackSuppressionDuration = 0
        change.set_status(ChangeForReaderStatusKind::Unacknowledged);

        RtpsChangeForReaderCacheChange::new(change.clone(), self.writer.writer_cache())
    }

    pub fn unsent_changes(&self) -> Vec<SequenceNumber> {
        // "return change IN this.changes_for_reader SUCH-THAT (change.status == UNSENT);"
        self.reader_proxy
            .changes_for_reader()
            .iter()
            .filter_map(|cc| {
                if cc.status() == ChangeForReaderStatusKind::Unsent {
                    Some(cc.sequence_number())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn requested_changes(&self) -> Vec<SequenceNumber> {
        // "return change IN this.changes_for_reader
        //      SUCH-THAT (change.status == REQUESTED);"
        let requested_changes_for_reader: Vec<_> = self
            .reader_proxy
            .changes_for_reader()
            .iter()
            .filter(|&change_for_reader| {
                change_for_reader.status == ChangeForReaderStatusKind::Requested
            })
            .collect();
        requested_changes_for_reader
            .iter()
            .map(|change_for_reader| change_for_reader.sequence_number)
            .collect()
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        // "FOR_EACH seq_num IN req_seq_num_set DO
        //     FIND change_for_reader IN this.changes_for_reader
        //          SUCH-THAT (change_for_reader.sequenceNumber==seq_num)
        //     change_for_reader.status := REQUESTED;
        // END"
        for &seq_num in req_seq_num_set {
            for change_for_reader in &mut self
                .reader_proxy
                .changes_for_reader_mut()
                .iter_mut()
                .filter(|change_for_reader| change_for_reader.sequence_number == seq_num)
            {
                change_for_reader.status = ChangeForReaderStatusKind::Requested;
            }
        }
    }

    pub fn unacked_changes(&self) -> Vec<SequenceNumber> {
        //"return change IN this.changes_for_reader
        //    SUCH-THAT (change.status == UNACKNOWLEDGED);"
        self.reader_proxy
            .changes_for_reader()
            .iter()
            .filter_map(|cc| {
                if cc.status == ChangeForReaderStatusKind::Unacknowledged {
                    Some(cc.sequence_number)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RtpsChangeForReader {
    status: ChangeForReaderStatusKind,
    is_relevant: bool,
    sequence_number: SequenceNumber,
}

impl RtpsChangeForReader {
    pub fn new(
        status: ChangeForReaderStatusKind,
        is_relevant: bool,
        sequence_number: SequenceNumber,
    ) -> Self {
        Self {
            status,
            is_relevant,
            sequence_number,
        }
    }

    pub fn status(&self) -> ChangeForReaderStatusKind {
        self.status
    }

    pub fn set_status(&mut self, status: ChangeForReaderStatusKind) {
        self.status = status;
    }

    pub fn is_relevant(&self) -> bool {
        self.is_relevant
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }
}

pub struct RtpsChangeForReaderCacheChange<'a> {
    change_for_reader: RtpsChangeForReader,
    cache_change: &'a RtpsWriterCacheChange,
}

impl<'a> RtpsChangeForReaderCacheChange<'a> {
    pub fn new(
        change_for_reader: RtpsChangeForReader,
        writer_cache: &'a WriterHistoryCache,
    ) -> Self {
        let cache_change = writer_cache
            .change_list()
            .iter()
            .find(|cc| cc.sequence_number() == change_for_reader.sequence_number)
            .unwrap();
        RtpsChangeForReaderCacheChange {
            change_for_reader,
            cache_change,
        }
    }

    pub fn cache_change(self) -> &'a RtpsWriterCacheChange {
        self.cache_change
    }

    pub fn is_relevant(&self) -> bool {
        self.change_for_reader.is_relevant
    }
}
