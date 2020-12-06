use crate::dds::implementation::rtps_publisher::RtpsPublisher;

/// The Publisher acts on the behalf of one or several DataWriter objects that belong to it. When it is informed of a change to the
/// data associated with one of its DataWriter objects, it decides when it is appropriate to actually send the data-update message.
/// In making this decision, it considers any extra information that goes with the data (timestamp, writer, etc.) as well as the QoS
/// of the Publisher and the DataWriter.
/// All operations except for the base-class operations set_qos, get_qos, set_listener, get_listener, enable, get_statuscondition,
/// create_datawriter, and delete_datawriter may return the value NOT_ENABLED.
pub struct Publisher(pub RtpsPublisher);