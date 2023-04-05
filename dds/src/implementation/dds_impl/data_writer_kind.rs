use super::{
    listener_data_writer::ListenerDataWriter, user_defined_data_writer::UserDefinedDataWriter,
};

pub enum DataWriterKind {
    UserDefined(UserDefinedDataWriter),
    Listener(ListenerDataWriter),
}
