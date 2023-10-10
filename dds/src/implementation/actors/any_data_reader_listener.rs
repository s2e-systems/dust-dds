use crate::{
    implementation::dds::dds_data_reader::DdsDataReader,
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::{data_reader::DataReader, data_reader_listener::DataReaderListener},
};

pub trait AnyDataReaderListener {
    fn trigger_on_data_available(&mut self, reader: DdsDataReader);
    fn trigger_on_sample_rejected(&mut self, reader: DdsDataReader, status: SampleRejectedStatus);
    fn trigger_on_liveliness_changed(
        &mut self,
        reader: DdsDataReader,
        status: LivelinessChangedStatus,
    );
    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DdsDataReader,
        status: RequestedDeadlineMissedStatus,
    );
    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DdsDataReader,
        status: RequestedIncompatibleQosStatus,
    );
    fn trigger_on_subscription_matched(
        &mut self,
        reader: DdsDataReader,
        status: SubscriptionMatchedStatus,
    );
    fn trigger_on_sample_lost(&mut self, reader: DdsDataReader, status: SampleLostStatus);
}

impl<Foo> AnyDataReaderListener for Box<dyn DataReaderListener<Foo> + Send> {
    fn trigger_on_data_available(&mut self, reader: DdsDataReader) {
        self.on_data_available(&DataReader::new(reader))
    }

    fn trigger_on_sample_rejected(&mut self, reader: DdsDataReader, status: SampleRejectedStatus) {
        self.on_sample_rejected(&DataReader::new(reader), status)
    }

    fn trigger_on_liveliness_changed(
        &mut self,
        reader: DdsDataReader,
        status: LivelinessChangedStatus,
    ) {
        self.on_liveliness_changed(&DataReader::new(reader), status)
    }

    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DdsDataReader,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.on_requested_deadline_missed(&DataReader::new(reader), status)
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DdsDataReader,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.on_requested_incompatible_qos(&DataReader::new(reader), status)
    }

    fn trigger_on_subscription_matched(
        &mut self,
        reader: DdsDataReader,
        status: SubscriptionMatchedStatus,
    ) {
        self.on_subscription_matched(&DataReader::new(reader), status)
    }

    fn trigger_on_sample_lost(&mut self, reader: DdsDataReader, status: SampleLostStatus) {
        self.on_sample_lost(&DataReader::new(reader), status)
    }
}
