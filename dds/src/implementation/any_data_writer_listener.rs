use crate::{
    dds_async::{data_writer::DataWriterAsync, data_writer_listener::DataWriterListenerAsync},
    infrastructure::status::{
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
    },
    runtime::executor::block_on,
};

pub trait AnyDataWriterListener: Send {
    fn trigger_on_publication_matched(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: PublicationMatchedStatus,
    );
    fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: OfferedIncompatibleQosStatus,
    );
    fn trigger_on_offered_deadline_missed(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: OfferedDeadlineMissedStatus,
    );
}

impl<'a, Foo> AnyDataWriterListener for Box<dyn DataWriterListenerAsync<'a, Foo = Foo> + Send + 'a>
where
    Foo: 'a,
{
    fn trigger_on_publication_matched(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: PublicationMatchedStatus,
    ) {
        block_on(self.on_publication_matched(the_writer.change_foo_type(), status))
    }

    fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: OfferedIncompatibleQosStatus,
    ) {
        block_on(self.on_offered_incompatible_qos(the_writer.change_foo_type(), status))
    }

    fn trigger_on_offered_deadline_missed(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: OfferedDeadlineMissedStatus,
    ) {
        block_on(self.on_offered_deadline_missed(the_writer.change_foo_type(), status))
    }
}
