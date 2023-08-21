use super::{
    writer_history_cache::WriterHistoryCache,
    messages::{
        overall_structure::RtpsSubmessageWriteKind,
        submessages::{
            heartbeat::HeartbeatSubmessageWrite, heartbeat_frag::HeartbeatFragSubmessageWrite,
        },
        types::{Count, FragmentNumber},
    },
    types::{EntityId, Guid, Locator, ReliabilityKind, SequenceNumber},
    utils::clock::{StdTimer, Timer, TimerConstructor},
};
use crate::infrastructure::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatMachine {
    count: Count,
    reader_id: EntityId,
    timer: StdTimer,
}
impl HeartbeatMachine {
    fn new(reader_id: EntityId) -> Self {
        HeartbeatMachine {
            count: 0,
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
        RtpsSubmessageWriteKind::Heartbeat(HeartbeatSubmessageWrite::new(
            false,
            false,
            self.reader_id,
            writer_id,
            first_sn,
            last_sn,
            self.count,
        ))
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
            count: 0,
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
        RtpsSubmessageWriteKind::HeartbeatFrag(HeartbeatFragSubmessageWrite::new(
            self.reader_id,
            writer_id,
            writer_sn,
            last_fragment_num,
            self.count,
        ))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsReaderProxy {
    remote_reader_guid: Guid,
    remote_group_entity_id: EntityId,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    highest_sent_seq_num: SequenceNumber,
    highest_acked_seq_num: SequenceNumber,
    requested_changes: Vec<SequenceNumber>,
    expects_inline_qos: bool,
    is_active: bool,
    last_received_acknack_count: Count,
    last_received_nack_frag_count: Count,
    heartbeat_machine: HeartbeatMachine,
    heartbeat_frag_machine: HeartbeatFragMachine,
    reliability: ReliabilityKind,
    first_relevant_sample_seq_num: SequenceNumber,
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
        first_relevant_sample_seq_num: SequenceNumber,
    ) -> Self {
        let heartbeat_machine = HeartbeatMachine::new(remote_reader_guid.entity_id());
        let heartbeat_frag_machine = HeartbeatFragMachine::new(remote_reader_guid.entity_id());
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            highest_sent_seq_num: SequenceNumber::from(0),
            highest_acked_seq_num: SequenceNumber::from(0),
            requested_changes: Vec::new(),
            expects_inline_qos,
            is_active,
            last_received_acknack_count: 0,
            last_received_nack_frag_count: 0,
            heartbeat_machine,
            heartbeat_frag_machine,
            reliability,
            first_relevant_sample_seq_num,
        }
    }

    pub fn remote_reader_guid(&self) -> Guid {
        self.remote_reader_guid
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_slice()
    }

    pub fn reliability(&self) -> ReliabilityKind {
        self.reliability
    }

    pub fn heartbeat_machine(&mut self) -> &mut HeartbeatMachine {
        &mut self.heartbeat_machine
    }

    pub fn heartbeat_frag_machine(&mut self) -> &mut HeartbeatFragMachine {
        &mut self.heartbeat_frag_machine
    }

    // //////////////   ReaderProxy operations defined in the Rtps Standard

    pub fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber) {
        if committed_seq_num > self.highest_acked_seq_num {
            self.highest_acked_seq_num = committed_seq_num
        }
    }

    pub fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        let next_requested_change = self.requested_changes.iter().min().cloned();

        if let Some(next_sn) = &next_requested_change {
            self.requested_changes.retain(|sn| sn != next_sn);
        }

        next_requested_change
    }

    pub fn next_unsent_change(
        &self,
        writer_history_cache: &WriterHistoryCache,
    ) -> Option<SequenceNumber> {
        //         unsent_changes :=
        // { changes SUCH_THAT change.sequenceNumber > this.highestSentChangeSN }
        //
        // IF unsent_changes == <empty> return SEQUENCE_NUMBER_INVALID
        // ELSE return MIN { unsent_changes.sequenceNumber }
        writer_history_cache
            .change_list()
            .map(|cc| cc.sequence_number())
            .filter(|cc_sn| cc_sn > &self.highest_sent_seq_num)
            .min()
    }

    pub fn unsent_changes(&self, writer_history_cache: &WriterHistoryCache) -> bool {
        // return this.next_unsent_change() != SEQUENCE_NUMBER_INVALID;
        self.next_unsent_change(writer_history_cache).is_some()
    }

    pub fn requested_changes(&self) -> Vec<SequenceNumber> {
        self.requested_changes.clone()
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        // "FOR_EACH seq_num IN req_seq_num_set DO
        //     FIND change_for_reader IN this.changes_for_reader
        //          SUCH-THAT (change_for_reader.sequenceNumber==seq_num)
        //     change_for_reader.status := REQUESTED;
        // END"
        for seq_num in req_seq_num_set {
            if !self.requested_changes.contains(seq_num) {
                self.requested_changes.push(*seq_num);
            }
        }
    }

    pub fn unacked_changes(&self, writer_history_cache: &WriterHistoryCache) -> bool {
        // highest_available_seq_num := MAX { change.sequenceNumber }
        // highest_acked_seq_num := MAX { this.acknowledged_changes }
        // return ( highest_available_seq_num > highest_acked_seq_num )

        let highest_available_seq_num = writer_history_cache
            .change_list()
            .map(|cc| cc.sequence_number())
            .max();

        match highest_available_seq_num {
            Some(highest_available_seq_num) => {
                highest_available_seq_num > self.highest_acked_seq_num
            }
            None => false,
        }
    }

    pub fn highest_sent_seq_num(&self) -> SequenceNumber {
        self.highest_sent_seq_num
    }

    pub fn set_highest_sent_seq_num(&mut self, seq_num: SequenceNumber) {
        if seq_num > self.highest_sent_seq_num {
            self.highest_sent_seq_num = seq_num;
        }
    }

    pub fn first_relevant_sample_seq_num(&self) -> SequenceNumber {
        self.first_relevant_sample_seq_num
    }

    pub fn set_first_relevant_sample_seq_num(&mut self, seq_num: SequenceNumber) {
        self.first_relevant_sample_seq_num = seq_num;
    }

    pub fn last_received_acknack_count(&self) -> Count {
        self.last_received_acknack_count
    }

    pub fn set_last_received_acknack_count(&mut self, count: Count) {
        self.last_received_acknack_count = count;
    }

    pub fn last_received_nack_frag_count(&self) -> Count {
        self.last_received_nack_frag_count
    }

    pub fn set_last_received_nack_frag_count(&mut self, count: Count) {
        self.last_received_nack_frag_count = count;
    }
}
