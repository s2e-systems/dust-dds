impl<'a, PSM: crate::PIM> RTPSReaderLocator<PSM> {
    pub fn produce_messages(
        &'a mut self,
        writer_cache: &'a impl RTPSHistoryCache<PSM>,
        send_data_to: &mut impl FnMut(&Locator<PSM>, PSM::DataSubmesage),
        send_gap_to: &mut impl FnMut(&Locator<PSM>, PSM::GapSubmessage),
    ) {
        while self
            .unsent_changes(writer_cache)
            .into_iter()
            .next()
            .is_some()
        {
            // Pushing state
            if let Some(seq_num) = self.next_unsent_change(writer_cache) {
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
                    let writer_sn = change.sequence_number;
                    // let inline_qos = change.inline_qos.clone();
                    let serialized_payload = &change.data_value;
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
                    send_data_to(self.locator(), data)
                } else {
                    // Send Gap submessage
                    let endianness_flag = true.into();
                    let reader_id = PSM::ENTITYID_UNKNOWN;
                    let writer_id = PSM::ENTITYID_UNKNOWN;
                    let gap_start = seq_num;
                    let gap_list = core::iter::empty().collect();
                    let gap = Gap::new(endianness_flag, reader_id, writer_id, gap_start, gap_list);
                    send_gap_to(self.locator(), gap)
                }
            }
        }
    }
}