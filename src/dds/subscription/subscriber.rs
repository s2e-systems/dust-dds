use crate::dds::implementation::rtps_subscriber::RtpsSubscriber;
/// A Subscriber is the object responsible for the actual reception of the data resulting from its subscriptions
///
/// A Subscriber acts on the behalf of one or several DataReader objects that are related to it. When it receives data (from the
/// other parts of the system), it builds the list of concerned DataReader objects, and then indicates to the application that data is
/// available, through its listener or by enabling related conditions. The application can access the list of concerned DataReader
/// objects through the operation get_datareaders and then access the data available though operations on the DataReader.
/// All operations except for the base-class operations set_qos, get_qos, set_listener, get_listener, enable, get_statuscondition,
/// and create_datareader may return the value NOT_ENABLED.
pub struct Subscriber(pub RtpsSubscriber);


    