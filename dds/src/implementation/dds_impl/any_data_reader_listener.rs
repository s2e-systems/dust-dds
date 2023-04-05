use crate::{
    implementation::utils::shared_object::DdsShared,
    subscription::{
        data_reader::{DataReader, DataReaderKind},
        data_reader_listener::DataReaderListener,
    },
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

use super::user_defined_data_reader::UserDefinedDataReaderImpl;

pub trait AnyDataReaderListener {
    fn trigger_on_data_available(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>);
    fn trigger_on_sample_rejected(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>);
    fn trigger_on_liveliness_changed(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>);
    fn trigger_on_requested_deadline_missed(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>);
    fn trigger_on_requested_incompatible_qos(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>);
    fn trigger_on_subscription_matched(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>);
    fn trigger_on_sample_lost(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>);
}

impl<Foo> AnyDataReaderListener for Box<dyn DataReaderListener<Foo = Foo> + Send + Sync>
where
    Foo: DdsType + for<'de> DdsDeserialize<'de> + 'static,
{
    fn trigger_on_data_available(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>) {
        self.on_data_available(&DataReader::new(DataReaderKind::UserDefined(
            reader.downgrade(),
        )))
    }

    fn trigger_on_sample_rejected(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>) {
        self.on_sample_rejected(
            &DataReader::new(DataReaderKind::UserDefined(reader.downgrade())),
            reader.get_sample_rejected_status(),
        )
    }

    fn trigger_on_liveliness_changed(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>) {
        self.on_liveliness_changed(
            &DataReader::new(DataReaderKind::UserDefined(reader.downgrade())),
            reader.get_liveliness_changed_status(),
        )
    }

    fn trigger_on_requested_deadline_missed(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>) {
        self.on_requested_deadline_missed(
            &DataReader::new(DataReaderKind::UserDefined(reader.downgrade())),
            reader.get_requested_deadline_missed_status(),
        )
    }

    fn trigger_on_requested_incompatible_qos(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>) {
        self.on_requested_incompatible_qos(
            &DataReader::new(DataReaderKind::UserDefined(reader.downgrade())),
            reader.get_requested_incompatible_qos_status(),
        )
    }

    fn trigger_on_subscription_matched(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>) {
        self.on_subscription_matched(
            &DataReader::new(DataReaderKind::UserDefined(reader.downgrade())),
            reader.get_subscription_matched_status(),
        )
    }

    fn trigger_on_sample_lost(&mut self, reader: &DdsShared<UserDefinedDataReaderImpl>) {
        self.on_sample_lost(
            &DataReader::new(DataReaderKind::UserDefined(reader.downgrade())),
            reader.get_sample_lost_status(),
        )
    }
}
