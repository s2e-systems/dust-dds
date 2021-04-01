use crate::{behavior::RTPSWriter, structure::RTPSHistoryCache, RtpsPsm};

pub struct RTPSReaderLocator<PSM: RtpsPsm> {
    locator: PSM::Locator,
    expects_inline_qos: bool,
    requested_changes: PSM::SequenceNumberSet,
}

impl<PSM: RtpsPsm> RTPSReaderLocator<PSM> {
    pub fn new(locator: PSM::Locator, expects_inline_qos: bool) -> Self {
        Self {
            locator,
            expects_inline_qos,
            requested_changes: core::iter::empty().collect(),
        }
    }

    pub fn locator(&self) -> &PSM::Locator {
        &self.locator
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    pub fn next_requested_change(&mut self) -> Option<PSM::SequenceNumber> {
        todo!()
    }

    pub fn next_unsent_change(&mut self) -> Option<PSM::SequenceNumber> {
        todo!()
    }

    pub fn requested_changes(&self) -> PSM::SequenceNumberSet {
        self.requested_changes.clone()
    }

    pub fn requested_changes_set(
        //<HistoryCache: RTPSHistoryCache<PSM = PSM>>
        &mut self,
        req_seq_num_set: PSM::SequenceNumberSet,
        // writer: &RTPSWriter<PSM, HistoryCache>,
    ) {
        self.requested_changes = self
            .requested_changes
            .clone()
            .into_iter()
            .chain(
                req_seq_num_set.into_iter(), // .filter(|seq_num| writer.writer_cache.get_change(seq_num).is_some()),
            )
            .collect();
    }

    pub fn unsent_changes(&self) -> PSM::SequenceNumberSet {
        todo!()
    }
}

// pub trait RTPSReaderLocator {
//     type PSM: RtpsPsm;
//     type HistoryCache: RTPSHistoryCache;

//     fn requested_changes(&self) -> <Self::PSM as RtpsPsm>::SequenceNumberSet;
//     fn unsent_changes(&self) -> <Self::PSM as RtpsPsm>::SequenceNumberSet;
//     fn next_requested_change(&mut self) -> Option<<Self::PSM as structure::Types>::SequenceNumber>;
//     fn next_unsent_change(&mut self) -> Option<<Self::PSM as structure::Types>::SequenceNumber>;
//     fn requested_changes_set(
//         &mut self,
//         req_seq_num_set: &[<Self::PSM as structure::Types>::SequenceNumber],
//         writer: &RTPSWriter<Self::PSM, Self::HistoryCache>,
//     );
// }

// impl dyn RTPSReaderLocator {
// fn pushing_state<
//     'a,
//     DataSubmessage: submessages::data_submessage::Data<
//         EntityId = EntityIdType,
//         SequenceNumber = SequenceNumberType,
//         ParameterId = ParameterIdType,
//         ParameterValue = ParameterValueType,
//         ParameterList = ParameterListType,
//         SerializedData = &'a [u8],
//     >,
//     GapSubmessage: submessages::gap_submessage::Gap<
//         EntityId = EntityIdType,
//         SequenceNumber = SequenceNumberType,
//         SequenceNumberList = SequenceNumberListType,
//     >,
// >(
//     &mut self,
//     the_writer: &'a RTPSWriter<
//         GuidPrefixType,
//         EntityIdType,
//         LocatorType,
//         LocatorListType,
//         DurationType,
//         SequenceNumberType,
//         InstanceHandleType,
//         DataType,
//         ParameterIdType,
//         ParameterValueType,
//         ParameterListType,
//         HistoryCacheType,
//     >,
// ) -> Option<DataSubmessage> {
//     //     // RL::can_send() is always true when this function is called
// //     // so we don't bother making an if here
// //     Self::transition_t4(reader_locator, writer)
//     self.transition_t4(the_writer)
// }

// fn transition_t4<
//     'a,
//     DataSubmessage: submessages::data_submessage::Data<
//         EntityId = EntityIdType,
//         SequenceNumber = SequenceNumberType,
//         ParameterId = ParameterIdType,
//         ParameterValue = ParameterValueType,
//         ParameterList = ParameterListType,
//         SerializedData = &'a [u8],
//     >,
//     GapSubmessage: submessages::gap_submessage::Gap<
//         EntityId = EntityIdType,
//         SequenceNumber = SequenceNumberType,
//         SequenceNumberList = SequenceNumberListType,
//     >,
// >(
//     &mut self,
//     the_writer: &'a RTPSWriter<
//         GuidPrefixType,
//         EntityIdType,
//         LocatorType,
//         LocatorListType,
//         DurationType,
//         SequenceNumberType,
//         InstanceHandleType,
//         DataType,
//         ParameterIdType,
//         ParameterValueType,
//         ParameterListType,
//         HistoryCacheType,
//     >,
// ) -> Option<DataSubmessage> {
//     if let Some(next_unsent_sequence_number) = self.next_unsent_change() {
//         if let Some(next_unsent_cache_change) = the_writer
//             .writer_cache
//             .get_change(&next_unsent_sequence_number)
//         {
//             Some(data_submessage_from_cache_change(
//                 next_unsent_cache_change,
//                 <EntityIdType as EntityId>::ENTITYID_UNKNOWN,
//             ))
//         } else {
//             let gap = GapSubmessage::new(
//                 true.into(),
//                 submessage_elements::EntityId {
//                     value: <EntityIdType as EntityId>::ENTITYID_UNKNOWN,
//                 },
//                 submessage_elements::EntityId {
//                     value: the_writer.endpoint.guid.entity_id,
//                 },
//                 submessage_elements::SequenceNumber {
//                     value: next_unsent_sequence_number,
//                 },
//                 submessage_elements::SequenceNumberSet {
//                     base: next_unsent_sequence_number,
//                     set: self.requested_changes(),
//                 },
//             );
//             todo!()
//         }
//     } else {
//         None
//     }
// }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::mock_psm::{MockLocator, MockPsm};

    #[test]
    fn requested_changes_set() {
        let mut reader_locator: RTPSReaderLocator<MockPsm> =
            RTPSReaderLocator::new(MockLocator, false);
        let req_seq_num_set = vec![1, 3, 5];
        reader_locator.requested_changes_set(req_seq_num_set.clone());

        assert_eq!(reader_locator.requested_changes, req_seq_num_set)
    }
}
