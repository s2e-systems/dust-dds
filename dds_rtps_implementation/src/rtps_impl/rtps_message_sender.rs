use super::rtps_participant_impl::RTPSParticipantImpl;

struct UdpRtpsMessageSender;

impl UdpRtpsMessageSender {
    pub fn send_data<PSM:super::PIM>(participant: &RTPSParticipantImpl<PSM>) {
        for writer_group in participant.writer_groups() {
            if let Some(writer_group_lock) = writer_group.try_lock() {
                for writer in writer_group_lock.writer_list() {
                    if let Some(mut writer_lock) = writer.try_lock() {
                        let (reader_locators, _writer_cache) =
                            writer_lock.locators_and_writer_cache();
                        for _reader_locator in reader_locators {
                            // reader_locator.produce_messages::<RtpsDataSubmessageImpl<PSM>,RtpsGapSubmessageImpl<PSM>>(
                            //     writer_cache,
                            //     &mut |_, _| {},
                            //     &mut |_, _| {},
                            // );
                        }
                    }
                }
            }
        }
    }

    // pub fn send_data(&self) {
    //     for writer in &self.writer_list {
    //         if let Some(mut writer) = writer.try_lock() {
    //             let (reader_locators, writer_cache) = writer.locators_and_writer_cache();
    //             // let writer_cache = writer.writer_cache;
    //             // let (ref mut reader_locators, writer_cache) = (&writer.reader_locators, &writer.writer_cache);
    //             // let submessages;
    //             // for reader_locator in &mut writer_ref.reader_locators{
    //                 // reader_locator.produce_messages(&writer_cache, &mut |_,_|{}, &mut |_,_|{});
    //             // }
    //             // writer.produce_messages(&mut |_, _| {}, &mut |_, _| {});
    //             // transport.write(submessages)
    //         }
    //     }
    // }
}
