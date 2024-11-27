use crate::{
    dds_async::{data_reader::DataReaderAsync, data_reader_listener::DataReaderListenerAsync},
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
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
    fn trigger_on_sample_rejected(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: SampleRejectedStatus,
    );

    fn trigger_on_subscription_matched(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: SubscriptionMatchedStatus,
    );

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: RequestedIncompatibleQosStatus,
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
    fn trigger_on_sample_rejected(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: SampleRejectedStatus,
    ) {
        block_on(self.on_sample_rejected(the_reader.change_foo_type(), status))
    }

    fn trigger_on_subscription_matched(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: SubscriptionMatchedStatus,
    ) {
        block_on(self.on_subscription_matched(the_reader.change_foo_type(), status))
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: RequestedIncompatibleQosStatus,
    ) {
        block_on(self.on_requested_incompatible_qos(the_reader.change_foo_type(), status))
    }
}
