use crate::types::{EntityId, Locator, SequenceNumber, GUID};

/// This struct is a non-standard addition to support the
/// behavior implementation on the StatefulReader
// struct ChangesFromWriter {
//     highest_processed_sequence_number: SequenceNumber,
//     unknown_changes: BTreeSet<SequenceNumber>,
//     lost_changes: BTreeSet<SequenceNumber>,
//     missing_changes: BTreeSet<SequenceNumber>,
//     irrelevant_changes: BTreeSet<SequenceNumber>,
// }

// pub struct WriterProxyBehavior {
//     pub must_send_ack: bool,
//     pub time_heartbeat_received: Instant,
//     pub ackanck_count: Count,
// }

pub trait WriterProxy {
    type CacheChangeRepresentation;

    fn remote_writer_guid(&self) -> GUID;
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
    fn data_max_size_serialized(&self) -> i32;
    fn changes_from_writer(&self) -> &[Self::CacheChangeRepresentation];
    fn remote_group_entity_id(&self) -> EntityId;

    fn new(
        remote_writer_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        remote_group_entity_id: EntityId,
    ) -> Self;

    /// This operation returns the maximum SequenceNumber_t among the changes_from_writer changes in the RTPS WriterProxy
    /// that are available for access by the DDS DataReader. The condition to make any CacheChange ‘a_change’ available
    /// for ‘access’ by the DDS DataReader is that there are no changes from the RTPS Writer with SequenceNumber_t smaller
    /// than or equal to a_change.sequenceNumber that have status MISSING or UNKNOWN. In other words, the available_changes_max
    /// and all previous changes are either RECEIVED or LOST.
    fn available_changes_max(&mut self) -> SequenceNumber;

    /// This operation modifies the status of a ChangeFromWriter to indicate that the CacheChange with the
    /// SequenceNumber_t ‘a_seq_num’ is irrelevant to the RTPS Reader.
    fn irrelevant_change_set(&mut self, a_seq_num: SequenceNumber);

    /// This operation modifies the status stored in ChangeFromWriter for any changes in the WriterProxy whose
    /// status is MISSING or UNKNOWN and have sequence numbers lower than ‘first_available_seq_num.’ The status of
    /// those changes is modified to LOST indicating that the changes are no longer available in the WriterHistoryCache
    /// of the RTPS Writer represented by the RTPS WriterProxy
    fn lost_changes_update(&mut self, first_available_seq_num: SequenceNumber);

    /// This operation returns the subset of changes for the WriterProxy that have status ‘MISSING.’
    /// The changes with status ‘MISSING’ represent the set of changes available in the HistoryCache of the RTPS Writer
    /// represented by the RTPS WriterProxy that have not been received by the RTPS Reader.
    fn missing_changes(&mut self) -> &[SequenceNumber];

    /// This operation modifies the status stored in ChangeFromWriter for any changes in the WriterProxy whose status is UNKNOWN
    /// and have sequence numbers smaller or equal to ‘last_available_seq_num.’ The status of those changes is modified
    /// from UNKNOWN to MISSING indicating that the changes are available at the WriterHistoryCache of the RTPS Writer represented
    /// by the RTPS WriterProxy but have not been received by the RTPS Reader
    fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber);

    /// This operation modifies the status of the ChangeFromWriter that refers to the CacheChange with the
    /// SequenceNumber_t ‘a_seq_num.’ The status of the change is set to ‘RECEIVED,’ indicating it has been received.
    fn received_change_set(&mut self, a_seq_num: SequenceNumber);
}
