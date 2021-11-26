use core::iter::FromIterator;

use crate::{
    messages::{
        submessage_elements::{
            EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSetSubmessageElement, SequenceNumberSubmessageElement,
            SerializedDataSubmessageElement,
        },
        submessages::{DataSubmessage, GapSubmessage},
    },
    structure::{
        history_cache::RtpsHistoryCacheGetChange,
        types::{ChangeKind, Locator, SequenceNumber, ENTITYID_UNKNOWN},
    },
};

#[derive(Debug)]
pub struct RtpsReaderLocator {
    pub locator: Locator,
    pub expects_inline_qos: bool,
}

impl RtpsReaderLocator {
    pub fn new(locator: Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
        }
    }
}

pub trait RtpsReaderLocatorOperations {
    type SequenceNumberVector;

    fn next_requested_change(&mut self) -> Option<SequenceNumber>;

    fn next_unsent_change(
        &mut self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Option<SequenceNumber>;

    fn requested_changes(&self) -> Self::SequenceNumberVector;

    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[SequenceNumber],
        last_change_sequence_number: &SequenceNumber,
    );

    fn unsent_changes(
        &self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Self::SequenceNumberVector;
}

impl<S> dyn RtpsReaderLocatorOperations<SequenceNumberVector = S>
where
    S: FromIterator<SequenceNumber>,
{
    pub fn send_unsent_data<'a, P, D>(
        &mut self,
        writer_cache: &'a impl RtpsHistoryCacheGetChange<'a, P, D>,
        last_change_sequence_number: SequenceNumber,
        send_data: &mut dyn FnMut(DataSubmessage<P, D>),
        send_gap: &mut dyn FnMut(GapSubmessage<S>),
    ) {
        while let Some(seq_num) = self.next_unsent_change(&last_change_sequence_number) {
            if let Some(change) = writer_cache.get_change(&seq_num) {
                let endianness_flag = true;
                let inline_qos_flag = true;
                let (data_flag, key_flag) = match change.kind {
                    ChangeKind::Alive => (true, false),
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
                        (false, true)
                    }
                    _ => todo!(),
                };
                let non_standard_payload_flag = false;
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: *change.writer_guid.entity_id(),
                };
                let writer_sn = SequenceNumberSubmessageElement {
                    value: change.sequence_number,
                };
                let inline_qos = ParameterListSubmessageElement {
                    parameter: change.inline_qos,
                };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: change.data_value,
                };
                let data_submessage = DataSubmessage {
                    endianness_flag,
                    inline_qos_flag,
                    data_flag,
                    key_flag,
                    non_standard_payload_flag,
                    reader_id,
                    writer_id,
                    writer_sn,
                    inline_qos,
                    serialized_payload,
                };
                send_data(data_submessage)
            } else {
                let endianness_flag = true;
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                let set = core::iter::empty().collect();
                let gap_list = SequenceNumberSetSubmessageElement { base: seq_num, set };
                let gap_submessage = GapSubmessage {
                    endianness_flag,
                    reader_id,
                    writer_id,
                    gap_start,
                    gap_list,
                };
                send_gap(gap_submessage)
            }
        }
    }
}
