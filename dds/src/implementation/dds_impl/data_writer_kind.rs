use super::user_defined_data_writer::UserDefinedDataWriter;

pub enum DataWriterKind {
    UserDefined(UserDefinedDataWriter),
    Listener,
}
