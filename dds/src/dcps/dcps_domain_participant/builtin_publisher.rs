use crate::{
    dcps::dcps_domain_participant::data_writer_entity::DataWriterEntity,
    rtps::{stateful_writer::RtpsStatefulWriter, stateless_writer::RtpsStatelessWriter},
};

pub struct BuiltinPublisher {
    pub dcps_participant_writer: DataWriterEntity<RtpsStatelessWriter>,
    pub dcps_topics_writer: DataWriterEntity<RtpsStatefulWriter>,
    pub dcps_publications_writer: DataWriterEntity<RtpsStatefulWriter>,
    pub dcps_subscriptions_writer: DataWriterEntity<RtpsStatefulWriter>,
    pub type_lookup_request_writer: DataWriterEntity<RtpsStatefulWriter>,
    pub type_lookup_reply_writer: DataWriterEntity<RtpsStatefulWriter>,
    pub dcps_participant_secure_writer: Option<DataWriterEntity<RtpsStatefulWriter>>,
    pub enabled: bool,
}

impl BuiltinPublisher {
    pub fn enable(&mut self) {
        self.dcps_participant_writer.enabled = true;
        for dw in self.stateful_data_writer_list_mut() {
            dw.enabled = true;
        }
        self.enabled = true;
    }

    pub fn stateful_data_writer_list(&self) -> [&DataWriterEntity<RtpsStatefulWriter>; 5] {
        [
            &self.dcps_topics_writer,
            &self.dcps_publications_writer,
            &self.dcps_subscriptions_writer,
            &self.type_lookup_request_writer,
            &self.type_lookup_reply_writer,
        ]
    }

    pub fn stateful_data_writer_list_mut(
        &mut self,
    ) -> [&mut DataWriterEntity<RtpsStatefulWriter>; 5] {
        [
            &mut self.dcps_topics_writer,
            &mut self.dcps_publications_writer,
            &mut self.dcps_subscriptions_writer,
            &mut self.type_lookup_request_writer,
            &mut self.type_lookup_reply_writer,
        ]
    }
}
