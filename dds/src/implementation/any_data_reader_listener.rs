use crate::{
    dds_async::{data_reader::DataReaderAsync, data_reader_listener::DataReaderListenerAsync},
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    runtime::executor::block_on,
};

pub trait AnyDataReaderListener: Send {
    fn trigger_on_data_available(&mut self, the_reader: DataReaderAsync<()>);
    fn trigger_on_requested_deadline_missed(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: RequestedDeadlineMissedStatus,
    );
}

impl<'a, Foo> AnyDataReaderListener for Box<dyn DataReaderListenerAsync<'a, Foo = Foo> + Send + 'a>
where
    Foo: 'a,
{
    fn trigger_on_data_available(&mut self, the_reader: DataReaderAsync<()>) {
        block_on(self.on_data_available(the_reader.change_foo_type()));
    }

    fn trigger_on_requested_deadline_missed(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: RequestedDeadlineMissedStatus,
    ) {
        block_on(self.on_requested_deadline_missed(the_reader.change_foo_type(), status))
    }
}
