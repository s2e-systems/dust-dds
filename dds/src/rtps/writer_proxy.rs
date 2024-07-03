use super::{
    messages::{
        overall_structure::Submessage,
        submessage_elements::{Data, FragmentNumberSet, SequenceNumberSet},
        submessages::{
            ack_nack::AckNackSubmessage, data::DataSubmessage, data_frag::DataFragSubmessage,
            info_destination::InfoDestinationSubmessage, nack_frag::NackFragSubmessage,
        },
        types::Count,
    },
    types::{EntityId, Guid, Locator, SequenceNumber},
};
use crate::implementation::{
    actor::ActorAddress,
    actors::message_sender_actor::{self, MessageSenderActor},
};
use std::{cmp::max, collections::HashMap, sync::Arc};

fn total_fragments_expected(data_frag_submessage: &DataFragSubmessage) -> u32 {
    let data_size = data_frag_submessage.data_size();
    let fragment_size = data_frag_submessage.fragment_size() as u32;
    let total_fragments_correction = if data_size % fragment_size == 0 { 0 } else { 1 };
    data_size / fragment_size + total_fragments_correction
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsWriterProxy {
    remote_writer_guid: Guid,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    data_max_size_serialized: Option<i32>,
    remote_group_entity_id: EntityId,
    first_available_seq_num: SequenceNumber,
    last_available_seq_num: SequenceNumber,
    highest_received_change_sn: SequenceNumber,
    must_send_acknacks: bool,
    last_received_heartbeat_count: Count,
    last_received_heartbeat_frag_count: Count,
    acknack_count: Count,
    nack_frag_count: Count,
    frag_buffer: HashMap<SequenceNumber, Vec<DataFragSubmessage>>,
}

impl RtpsWriterProxy {
    pub fn new(
        remote_writer_guid: Guid,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        data_max_size_serialized: Option<i32>,
        remote_group_entity_id: EntityId,
    ) -> Self {
        Self {
            remote_writer_guid,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            data_max_size_serialized,
            remote_group_entity_id,
            first_available_seq_num: 1,
            last_available_seq_num: 0,
            highest_received_change_sn: 0,
            must_send_acknacks: false,
            last_received_heartbeat_count: 0,
            last_received_heartbeat_frag_count: 0,
            acknack_count: 0,
            nack_frag_count: 0,
            frag_buffer: HashMap::new(),
        }
    }

    pub fn push_data_frag(&mut self, submessage: DataFragSubmessage) {
        let frag_bug_seq_num = self.frag_buffer.entry(submessage.writer_sn()).or_default();
        if !frag_bug_seq_num.contains(&submessage) {
            frag_bug_seq_num.push(submessage);
        }
    }

    pub fn reconstruct_data_from_frag(
        &mut self,
        seq_num: SequenceNumber,
    ) -> Option<DataSubmessage> {
        if let Some(seq_num_frag) = self.frag_buffer.get(&seq_num) {
            let total_fragments_expected = total_fragments_expected(&seq_num_frag[0]);

            let mut total_fragments = 0;
            for frag_seq_num in seq_num_frag {
                total_fragments += frag_seq_num.fragments_in_submessage() as u32;
            }

            if total_fragments == total_fragments_expected {
                let mut frag_seq_num_list = self.frag_buffer.remove(&seq_num).expect("Must exist");
                frag_seq_num_list.sort_by_key(|k| k.fragment_starting_num());

                let inline_qos_flag = frag_seq_num_list[0].inline_qos_flag();
                let data_flag = !frag_seq_num_list[0].key_flag();
                let key_flag = frag_seq_num_list[0].key_flag();
                let non_standard_payload_flag = false;
                let writer_id = self.remote_writer_guid.entity_id();
                let reader_id = frag_seq_num_list[0].reader_id();
                let writer_sn = seq_num;
                let inline_qos = frag_seq_num_list[0].inline_qos().clone();
                let mut data = Vec::new();
                for frag in frag_seq_num_list {
                    data.extend_from_slice(frag.serialized_payload().as_ref());
                }

                Some(DataSubmessage::new(
                    inline_qos_flag,
                    data_flag,
                    key_flag,
                    non_standard_payload_flag,
                    reader_id,
                    writer_id,
                    writer_sn,
                    inline_qos,
                    Data::new(Arc::from(data)),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn remote_writer_guid(&self) -> Guid {
        self.remote_writer_guid
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.unicast_locator_list.as_ref()
    }

    pub fn available_changes_max(&self) -> SequenceNumber {
        // The condition to make any CacheChange 'a_change' available for 'access' by the DDS DataReader is that there are no changes
        // from the RTPS Writer with SequenceNumber_t smaller than or equal to a_change.sequenceNumber that have status MISSING or UNKNOWN.

        max(
            self.first_available_seq_num - 1,
            self.highest_received_change_sn,
        )
    }

    pub fn irrelevant_change_set(&mut self, a_seq_num: SequenceNumber) {
        // This operation modifies the status of a ChangeFromWriter to indicate that the CacheChange with the
        // SequenceNumber_t 'a_seq_num' is irrelevant to the RTPS Reader. Logical action in the virtual machine:
        // FIND change FROM this.changes_from_writer SUCH-THAT
        // (change.sequenceNumber == a_seq_num);
        // change.status := RECEIVED; change.is_relevant := FALSE;
        if a_seq_num > self.highest_received_change_sn {
            self.highest_received_change_sn = a_seq_num;
        }
    }

    pub fn lost_changes_update(&mut self, first_available_seq_num: SequenceNumber) {
        // FOREACH change IN this.changes_from_writer
        // SUCH-THAT ( change.status == UNKNOWN OR change.status == MISSING
        // AND seq_num < first_available_seq_num ) DO {
        // change.status := LOST;
        // }
        self.first_available_seq_num = first_available_seq_num;
    }

    pub fn missing_changes(&self) -> impl Iterator<Item = SequenceNumber> {
        // The changes with status 'MISSING' represent the set of changes available in the HistoryCache of the RTPS Writer
        // represented by the RTPS WriterProxy that have not been received by the RTPS Reader.
        // return { change IN this.changes_from_writer SUCH-THAT change.status == MISSING};

        // The highest sequence number of all present
        let highest_number = max(self.last_available_seq_num, self.highest_received_change_sn);

        // Changes below first_available_seq_num are LOST (or RECEIVED, but in any case not MISSING) and above last_available_seq_num are unknown.
        // In between those two numbers, every change that is not RECEIVED or IRRELEVANT is MISSING
        let first_missing_change = max(
            self.first_available_seq_num,
            self.highest_received_change_sn + 1,
        );
        first_missing_change..=highest_number
    }

    pub fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber) {
        // FOREACH change IN this.changes_from_writer
        // SUCH-THAT ( change.status == UNKNOWN
        // AND seq_num <= last_available_seq_num ) DO {
        // change.status := MISSING;
        // }
        self.last_available_seq_num = last_available_seq_num;
    }

    pub fn received_change_set(&mut self, a_seq_num: SequenceNumber) {
        // FIND change FROM this.changes_from_writer
        //     SUCH-THAT change.sequenceNumber == a_seq_num;
        // change.status := RECEIVED
        if a_seq_num > self.highest_received_change_sn {
            self.highest_received_change_sn = a_seq_num;
        }
    }

    pub fn set_must_send_acknacks(&mut self, must_send_acknacks: bool) {
        self.must_send_acknacks = must_send_acknacks;
    }

    pub fn must_send_acknacks(&self) -> bool {
        self.must_send_acknacks
    }

    pub fn last_received_heartbeat_count(&self) -> Count {
        self.last_received_heartbeat_count
    }

    pub fn set_last_received_heartbeat_count(&mut self, last_received_heartbeat_count: Count) {
        self.last_received_heartbeat_count = last_received_heartbeat_count;
    }

    pub fn set_last_received_heartbeat_frag_count(
        &mut self,
        last_received_heartbeat_frag_count: Count,
    ) {
        self.last_received_heartbeat_frag_count = last_received_heartbeat_frag_count;
    }

    pub fn acknack_count(&self) -> Count {
        self.acknack_count
    }

    pub fn increment_acknack_count(&mut self) {
        self.acknack_count = self.acknack_count.wrapping_add(1);
    }

    pub fn send_message(
        &mut self,
        reader_guid: &Guid,
        message_sender_actor: &ActorAddress<MessageSenderActor>,
    ) {
        if self.must_send_acknacks() || !self.missing_changes().count() == 0 {
            self.set_must_send_acknacks(false);
            self.increment_acknack_count();

            let info_dst_submessage =
                InfoDestinationSubmessage::new(self.remote_writer_guid().prefix());

            let acknack_submessage = AckNackSubmessage::new(
                true,
                reader_guid.entity_id(),
                self.remote_writer_guid().entity_id(),
                SequenceNumberSet::new(
                    self.available_changes_max() + 1,
                    self.missing_changes().take(256),
                ),
                self.acknack_count(),
            );

            let mut submessages: Vec<Box<dyn Submessage + Send>> =
                vec![Box::new(info_dst_submessage), Box::new(acknack_submessage)];

            for (seq_num, owning_data_frag_list) in self.frag_buffer.iter() {
                let total_fragments_expected = total_fragments_expected(&owning_data_frag_list[0]);
                let mut missing_fragment_number = Vec::new();
                for fragment_number in 1..=total_fragments_expected {
                    if !owning_data_frag_list.iter().any(|x| {
                        fragment_number >= x.fragment_starting_num()
                            && fragment_number
                                < x.fragment_starting_num() + (x.fragments_in_submessage() as u32)
                    }) {
                        missing_fragment_number.push(fragment_number)
                    }
                }

                if !missing_fragment_number.is_empty() {
                    self.nack_frag_count = self.nack_frag_count.wrapping_add(1);
                    let nack_frag_submessage = NackFragSubmessage::new(
                        reader_guid.entity_id(),
                        self.remote_writer_guid().entity_id(),
                        *seq_num,
                        FragmentNumberSet::new(
                            missing_fragment_number[0],
                            missing_fragment_number.into_iter(),
                        ),
                        self.nack_frag_count,
                    );

                    submessages.push(Box::new(nack_frag_submessage))
                }
            }

            message_sender_actor
                .send_actor_mail(message_sender_actor::WriteMessage {
                    submessages,
                    destination_locator_list: self.unicast_locator_list().to_vec(),
                })
                .ok();
        }
    }

    pub fn is_historical_data_received(&self) -> bool {
        let at_least_one_heartbeat_received = self.last_received_heartbeat_count > 0;
        at_least_one_heartbeat_received && self.missing_changes().count() == 0
    }
}
