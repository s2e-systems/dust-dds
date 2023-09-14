use crate::{
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::{data_reader::DataReader, data_reader_listener::DataReaderListener},
};

use super::nodes::{DataReaderNode, DataReaderNodeKind};

pub trait AnyDataReaderListener {
    fn trigger_on_data_available(&mut self, reader: DataReaderNode);
    fn trigger_on_sample_rejected(&mut self, reader: DataReaderNode, status: SampleRejectedStatus);
    fn trigger_on_liveliness_changed(
        &mut self,
        reader: DataReaderNode,
        status: LivelinessChangedStatus,
    );
    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DataReaderNode,
        status: RequestedDeadlineMissedStatus,
    );
    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DataReaderNode,
        status: RequestedIncompatibleQosStatus,
    );
    fn trigger_on_subscription_matched(
        &mut self,
        reader: DataReaderNode,
        status: SubscriptionMatchedStatus,
    );
    fn trigger_on_sample_lost(&mut self, reader: DataReaderNode, status: SampleLostStatus);
}

impl<Foo> AnyDataReaderListener for Box<dyn DataReaderListener<Foo> + Send + Sync> {
    fn trigger_on_data_available(&mut self, reader: DataReaderNode) {
        self.on_data_available(&DataReader::new(DataReaderNodeKind::Listener(reader)))
    }

    fn trigger_on_sample_rejected(&mut self, reader: DataReaderNode, status: SampleRejectedStatus) {
        self.on_sample_rejected(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }

    fn trigger_on_liveliness_changed(
        &mut self,
        reader: DataReaderNode,
        status: LivelinessChangedStatus,
    ) {
        self.on_liveliness_changed(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }

    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: DataReaderNode,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.on_requested_deadline_missed(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: DataReaderNode,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.on_requested_incompatible_qos(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }

    fn trigger_on_subscription_matched(
        &mut self,
        reader: DataReaderNode,
        status: SubscriptionMatchedStatus,
    ) {
        self.on_subscription_matched(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }

    fn trigger_on_sample_lost(&mut self, reader: DataReaderNode, status: SampleLostStatus) {
        self.on_sample_lost(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }
}
