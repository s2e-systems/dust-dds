use crate::{
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::{data_reader::DataReader, data_reader_listener::DataReaderListener},
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::{
    node_kind::DataReaderNodeKind, node_user_defined_data_reader::UserDefinedDataReaderNode,
};

pub trait AnyDataReaderListener {
    fn trigger_on_data_available(&mut self, reader: UserDefinedDataReaderNode);
    fn trigger_on_sample_rejected(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: SampleRejectedStatus,
    );
    fn trigger_on_liveliness_changed(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: LivelinessChangedStatus,
    );
    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: RequestedDeadlineMissedStatus,
    );
    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: RequestedIncompatibleQosStatus,
    );
    fn trigger_on_subscription_matched(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: SubscriptionMatchedStatus,
    );
    fn trigger_on_sample_lost(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: SampleLostStatus,
    );
}

impl<Foo> AnyDataReaderListener for Box<dyn DataReaderListener<Foo = Foo> + Send + Sync>
where
    Foo: DdsType + for<'de> DdsDeserialize<'de> + 'static,
{
    fn trigger_on_data_available(&mut self, reader: UserDefinedDataReaderNode) {
        self.on_data_available(&DataReader::new(DataReaderNodeKind::Listener(reader)))
    }

    fn trigger_on_sample_rejected(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: SampleRejectedStatus,
    ) {
        self.on_sample_rejected(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }

    fn trigger_on_liveliness_changed(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: LivelinessChangedStatus,
    ) {
        self.on_liveliness_changed(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }

    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.on_requested_deadline_missed(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.on_requested_incompatible_qos(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }

    fn trigger_on_subscription_matched(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: SubscriptionMatchedStatus,
    ) {
        self.on_subscription_matched(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }

    fn trigger_on_sample_lost(
        &mut self,
        reader: UserDefinedDataReaderNode,
        status: SampleLostStatus,
    ) {
        self.on_sample_lost(
            &DataReader::new(DataReaderNodeKind::Listener(reader)),
            status,
        )
    }
}
