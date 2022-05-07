use crate::data_representation_builtin_endpoints::{
    discovered_reader_data::DiscoveredReaderData, discovered_writer_data::DiscoveredWriterData,
    spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
};

pub trait AddMatchedParticipant {
    fn add_matched_participant(&self, discovered_participant_data: &SpdpDiscoveredParticipantData);
}

pub trait AddMatchedWriter {
    fn add_matched_writer(&self, discovered_writer_data: &DiscoveredWriterData);
}

pub trait AddMatchedReader {
    fn add_matched_reader(&self, discovered_reader_data: &DiscoveredReaderData);
}
