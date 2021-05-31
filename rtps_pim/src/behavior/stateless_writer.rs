use crate::{
    behavior::RTPSWriter,
    messages::types::ParameterIdType,
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
    PSM: EntityIdType
        + DataType
        + GuidPrefixType
        + InstanceHandleType
        + LocatorType
        + SequenceNumberType
        + ParameterIdType
        + GUIDType<PSM>
        + ParameterListType<PSM>,
    CacheChange: RTPSCacheChange<PSM> + 'a,
>(
    reader_locator: &'a mut impl RTPSReaderLocator<PSM>,
    writer_cache: &'a impl RTPSHistoryCache<PSM, CacheChange = CacheChange>,
    mut send_data_to: impl FnMut(&'a CacheChange),
    mut send_gap_to: impl FnMut(&PSM::SequenceNumber),
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
                    send_data_to(change)
                } else {
                    send_gap_to(&seq_num)
                }
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

// }
