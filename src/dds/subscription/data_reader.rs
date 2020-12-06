use crate::types::DDSType;
use crate::dds::implementation::rtps_data_reader::RtpsDataReader;

/// A DataReader allows the application (1) to declare the data it wishes to receive (i.e., make a subscription) and (2) to access the
/// data received by the attached Subscriber.
///
/// A DataReader refers to exactly one TopicDescription (either a Topic, a ContentFilteredTopic, or a MultiTopic) that identifies
/// the data to be read. The subscription has a unique resulting type. The data-reader may give access to several instances of the
/// resulting type, which can be distinguished from each other by their key (as described in 2.2.1.2.2, Overall Conceptual Model).
///
/// All operations except for the base-class operations set_qos, get_qos, set_listener, get_listener, enable, and
/// get_statuscondition may return the error NOT_ENABLED.
/// All sample-accessing operations, namely all variants of read, take may return the error PRECONDITION_NOT_MET. The
/// circumstances that result on this are described in 2.2.2.5.2.8.
pub struct DataReader<T: DDSType>(pub RtpsDataReader<T>);