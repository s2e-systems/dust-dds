use super::{
    messages::{
        overall_structure::RtpsSubmessageWriteKind,
        submessages::{
            ack_nack::AckNackSubmessageRead, heartbeat::HeartbeatSubmessageWrite,
            heartbeat_frag::HeartbeatFragSubmessageWrite, nack_frag::NackFragSubmessageRead,
        },
        types::{Count, FragmentNumber},
    },
    types::{
        DurabilityKind, EntityId, ExpectsInlineQos, Guid, Locator, ReliabilityKind, SequenceNumber,
    },
    utils::clock::{StdTimer, Timer, TimerConstructor},
    writer::RtpsWriter,
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
        RtpsSubmessageWriteKind::HeartbeatFrag(HeartbeatFragSubmessageWrite::new(
            self.reader_id,
            writer_id,
            writer_sn,
            last_fragment_num,
            self.count,
        ))
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
    highest_sent_seq_num: SequenceNumber,
    highest_acked_seq_num: SequenceNumber,
    requested_changes: Vec<SequenceNumber>,
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
            highest_sent_seq_num: SequenceNumber::new(0),
            highest_acked_seq_num: SequenceNumber::new(0),
            requested_changes: Vec::new(),
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

    pub fn changes_for_reader(&self) -> &[RtpsChangeForReader] {
        self.changes_for_reader.as_slice()
    }

    pub fn changes_for_reader_mut(&mut self) -> &mut Vec<RtpsChangeForReader> {
        &mut self.changes_for_reader
    }

    pub fn durability(&self) -> DurabilityKind {
        self.durability
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

    pub fn writer(&self) -> &'a RtpsWriter {
        self.writer
    }

    pub fn remote_reader_guid(&self) -> Guid {
        self.reader_proxy.remote_reader_guid
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.reader_proxy.unicast_locator_list.as_slice()
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
        if committed_seq_num > self.reader_proxy.highest_acked_seq_num {
            self.reader_proxy.highest_acked_seq_num = committed_seq_num
        }
    }

    pub fn next_requested_change(&mut self) -> Option<SequenceNumber> {
        let next_requested_change = self.reader_proxy.requested_changes.iter().min().cloned();

        if let Some(next_sn) = &next_requested_change {
            self.reader_proxy
                .requested_changes
                .retain(|sn| sn != next_sn);
        }

        next_requested_change
    }

    pub fn next_unsent_change(&self) -> Option<SequenceNumber> {
        //         unsent_changes :=
        // { changes SUCH_THAT change.sequenceNumber > this.highestSentChangeSN }
        //
        // IF unsent_changes == <empty> return SEQUENCE_NUMBER_INVALID
        // ELSE return MIN { unsent_changes.sequenceNumber }
        self.writer
            .change_list()
            .iter()
            .map(|cc| cc.sequence_number())
            .filter(|cc_sn| cc_sn > &self.reader_proxy.highest_sent_seq_num)
            .min()
    }

    pub fn unsent_changes(&self) -> bool {
        // return this.next_unsent_change() != SEQUENCE_NUMBER_INVALID;
        self.next_unsent_change().is_some()
    }

    pub fn requested_changes(&self) -> Vec<SequenceNumber> {
        self.reader_proxy.requested_changes.clone()
    }

    pub fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]) {
        // "FOR_EACH seq_num IN req_seq_num_set DO
        //     FIND change_for_reader IN this.changes_for_reader
        //          SUCH-THAT (change_for_reader.sequenceNumber==seq_num)
        //     change_for_reader.status := REQUESTED;
        // END"
        for seq_num in req_seq_num_set {
            if !self.reader_proxy.requested_changes.contains(seq_num) {
                self.reader_proxy.requested_changes.push(*seq_num);
            }
        }
    }

    pub fn unacked_changes(&self) -> bool {
        // highest_available_seq_num := MAX { change.sequenceNumber }
        // highest_acked_seq_num := MAX { this.acknowledged_changes }
        // return ( highest_available_seq_num > highest_acked_seq_num )

        let highest_available_seq_num = self
            .writer
            .change_list()
            .iter()
            .map(|cc| cc.sequence_number())
            .max();

        match highest_available_seq_num {
            Some(highest_available_seq_num) => {
                highest_available_seq_num > self.reader_proxy.highest_acked_seq_num
            }
            None => false,
        }
    }

    pub fn highest_sent_seq_num(&self) -> SequenceNumber {
        self.reader_proxy.highest_sent_seq_num
    }

    pub fn set_highest_sent_seq_num(&mut self, seq_num: SequenceNumber) {
        if seq_num > self.reader_proxy.highest_sent_seq_num {
            self.reader_proxy.highest_sent_seq_num = seq_num;
        }
    }

    pub fn receive_acknack(&mut self, acknack_submessage: &AckNackSubmessageRead) {
        match self.reader_proxy.reliability {
            ReliabilityKind::BestEffort => (),
            ReliabilityKind::Reliable => {
                if acknack_submessage.count() > self.reader_proxy.last_received_acknack_count {
                    self.acked_changes_set(acknack_submessage.reader_sn_state().base - 1);
                    self.requested_changes_set(acknack_submessage.reader_sn_state().set.as_ref());

                    self.reader_proxy.last_received_acknack_count = acknack_submessage.count();
                }
            }
        }
    }

    pub fn receive_nack_frag(&mut self, nack_frag_submessage: &NackFragSubmessageRead) {
        match self.reader_proxy.reliability {
            ReliabilityKind::BestEffort => (),
            ReliabilityKind::Reliable => {
                if nack_frag_submessage.count() > self.reader_proxy.last_received_nack_frag_count {
                    self.requested_changes_set(&[nack_frag_submessage.writer_sn()]);
                    self.reader_proxy.last_received_nack_frag_count = nack_frag_submessage.count();
                }
            }
        }
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
