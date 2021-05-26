use super::rtps_participant_impl::RTPSParticipantImpl;

struct UdpRtpsMessageSender;

impl UdpRtpsMessageSender {
    pub fn send_data<PSM: super::PIM>(participant: &RTPSParticipantImpl<PSM>) {
        for writer_group in participant.writer_groups() {
            if let Some(writer_group_lock) = writer_group.try_lock() {
                for writer in writer_group_lock.writer_list() {
                    if let Some(mut writer_lock) = writer.try_lock() {
                        let (reader_locators, writer_cache) =
                            writer_lock.locators_and_writer_cache();
                        for reader_locator in reader_locators {
                            let mut data_vec: Vec<PSM::DataSubmesage> = Vec::new();
                            let mut gap_vec: Vec<PSM::GapSubmessage> = Vec::new();
                            rust_rtps_pim::behavior::stateless_writer::produce_messages(
                                reader_locator,
                                writer_cache,
                                &mut |_locator, data| data_vec.push(data),
                                &mut |_locator, gap| gap_vec.push(gap),
                            );
                        }
                    }
                }
            }
        }
    }
}
