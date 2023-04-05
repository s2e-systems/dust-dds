use crate::{
    publication::{data_writer::DataWriter, data_writer_listener::DataWriterListener},
    topic_definition::type_support::{DdsSerialize, DdsType},
};

use super::listener_data_writer::ListenerDataWriter;

pub trait AnyDataWriterListener {
    fn trigger_on_liveliness_lost(&mut self, _the_writer: ListenerDataWriter);
    fn trigger_on_offered_deadline_missed(&mut self, _the_writer: ListenerDataWriter);
    fn trigger_on_offered_incompatible_qos(&mut self, _the_writer: ListenerDataWriter);
    fn trigger_on_publication_matched(&mut self, _the_writer: ListenerDataWriter);
}

impl<Foo> AnyDataWriterListener for Box<dyn DataWriterListener<Foo = Foo> + Send + Sync>
where
    Foo: DdsType + DdsSerialize + 'static,
{
    fn trigger_on_liveliness_lost(&mut self, the_writer: ListenerDataWriter) {
        todo!()
        // self.on_liveliness_lost(
        //     &DataWriter::new(the_writer.downgrade()),
        //     the_writer.get_liveliness_lost_status(),
        // );
    }

    fn trigger_on_offered_deadline_missed(&mut self, the_writer: ListenerDataWriter) {
        todo!()
        // self.on_offered_deadline_missed(
        //     &DataWriter::new(the_writer.downgrade()),
        //     the_writer.get_offered_deadline_missed_status(),
        // );
    }

    fn trigger_on_offered_incompatible_qos(&mut self, the_writer: ListenerDataWriter) {
        todo!()
        // self.on_offered_incompatible_qos(
        //     &DataWriter::new(the_writer.downgrade()),
        //     the_writer.get_offered_incompatible_qos_status(),
        // );
    }

    fn trigger_on_publication_matched(&mut self, the_writer: ListenerDataWriter) {
        todo!()
        // self.on_publication_matched(
        //     &DataWriter::new(the_writer.downgrade()),
        //     the_writer.get_publication_matched_status(),
        // )
    }
}
