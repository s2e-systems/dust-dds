use crate::{
    behavior::RTPSWriter,
    messages::{
        submessages::{Data, Gap},
        types::{ParameterIdType, SubmessageFlagType, SubmessageKindType},
    },
    structure::{
        types::{
            DataType, EntityIdType, GUIDType, GuidPrefixType, InstanceHandleType, LocatorType,
            ParameterListType, SequenceNumberType,
        },
        RTPSCacheChange, RTPSHistoryCache,
    },
};

use super::types::DurationType;
pub trait RTPSReaderLocator<PSM: LocatorType + SequenceNumberType> {
    type SequenceNumberVector: IntoIterator<Item = PSM::SequenceNumber>;

    fn locator(&self) -> &PSM::Locator;

    fn expects_inline_qos(&self) -> bool;

    fn next_requested_change(&mut self) -> Option<PSM::SequenceNumber>;

    fn next_unsent_change(
        &mut self,
        last_change_sequence_number: PSM::SequenceNumber,
    ) -> Option<PSM::SequenceNumber>;

    fn requested_changes(&self) -> Self::SequenceNumberVector;

    fn requested_changes_set(
        &mut self,
        req_seq_num_set: Self::SequenceNumberVector,
        last_change_sequence_number: PSM::SequenceNumber,
    );

    fn unsent_changes(
        &self,
        last_change_sequence_number: PSM::SequenceNumber,
    ) -> Self::SequenceNumberVector;
}

pub trait RTPSStatelessWriter<
    PSM: GuidPrefixType
        + EntityIdType
        + DurationType
        + DataType
        + InstanceHandleType
        + LocatorType
        + SequenceNumberType
        + GUIDType<PSM>
        + ParameterIdType
        + ParameterListType<PSM>,
>: RTPSWriter<PSM>
{
    type ReaderLocatorType: RTPSReaderLocator<PSM>;

    fn reader_locators(&self) -> &[Self::ReaderLocatorType];

    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorType);

    fn reader_locator_remove(&mut self, a_locator: &PSM::Locator);

    fn unsent_changes_reset(&mut self);
}

pub fn produce_messages<
    'a,
    PSM: SubmessageKindType
        + SubmessageFlagType
        + EntityIdType
        + DataType
        + GuidPrefixType
        + InstanceHandleType
        + LocatorType
        + SequenceNumberType
        + ParameterIdType
        + GUIDType<PSM>
        + ParameterListType<PSM>,
    DataSubmesage: Data<PSM>,
    GapSubmessage: Gap<PSM>,
>(
    reader_locator: &'a mut impl RTPSReaderLocator<PSM>,
    writer_cache: &'a impl RTPSHistoryCache<PSM>,
    mut send_data_to: impl FnMut(&PSM::Locator, DataSubmesage),
    mut send_gap_to: impl FnMut(&PSM::Locator, GapSubmessage),
) {
    if let Some(highest_sequence_number) = writer_cache.get_seq_num_max() {
        while reader_locator
            .unsent_changes(*highest_sequence_number)
            .into_iter()
            .next()
            .is_some()
        {
            // Pushing state
            if let Some(seq_num) = reader_locator.next_unsent_change(*highest_sequence_number) {
                // Transition T4
                if let Some(change) = writer_cache.get_change(&seq_num) {
                    // Send Data submessage
                    let endianness_flag = true.into();
                    let inline_qos_flag = false.into();
                    let data_flag = true.into();
                    let key_flag = false.into();
                    let non_standard_payload_flag = false.into();
                    let reader_id = PSM::ENTITYID_UNKNOWN;
                    let writer_id = PSM::ENTITYID_UNKNOWN;
                    let writer_sn = *change.sequence_number();
                    // let inline_qos = change.inline_qos.clone();
                    let serialized_payload = change.data_value();
                    let data = Data::new(
                        endianness_flag,
                        inline_qos_flag,
                        data_flag,
                        key_flag,
                        non_standard_payload_flag,
                        reader_id,
                        writer_id,
                        writer_sn,
                        // inline_qos,
                        serialized_payload,
                    );
                    send_data_to(reader_locator.locator(), data)
                } else {
                    // Send Gap submessage
                    let endianness_flag = true.into();
                    let reader_id = PSM::ENTITYID_UNKNOWN;
                    let writer_id = PSM::ENTITYID_UNKNOWN;
                    let gap_start = seq_num;
                    let gap_list = &[]; //core::iter::empty().collect();
                    let gap = Gap::new(endianness_flag, reader_id, writer_id, gap_start, gap_list);
                    send_gap_to(reader_locator.locator(), gap)
                }
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

// }
