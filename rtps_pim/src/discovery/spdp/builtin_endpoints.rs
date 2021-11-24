use crate::{
    behavior::{
        reader::reader::RtpsReader, types::DURATION_ZERO,
        writer::stateless_writer::RtpsStatelessWriter,
    },
    structure::{
        history_cache::RtpsHistoryCacheConstructor,
        types::{
            EntityId, Guid, GuidPrefix, ReliabilityKind, TopicKind, BUILT_IN_READER_WITH_KEY,
            BUILT_IN_WRITER_WITH_KEY,
        },
    },
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_READER_WITH_KEY);

pub struct SpdpBuiltinParticipantWriter;

impl SpdpBuiltinParticipantWriter {
    pub fn create<L, C, R>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: L,
        multicast_locator_list: L,
    ) -> RtpsStatelessWriter<L, C, R>
    where
        C: RtpsHistoryCacheConstructor,
        R: Default,
    {
        let spdp_builtin_participant_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);

        RtpsStatelessWriter::new(
            spdp_builtin_participant_writer_guid,
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            unicast_locator_list,
            multicast_locator_list,
            true,
            DURATION_ZERO,
            DURATION_ZERO,
            DURATION_ZERO,
            None,
        )
    }
}

pub struct SpdpBuiltinParticipantReader;

impl SpdpBuiltinParticipantReader {
    pub fn create<L, C>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: L,
        multicast_locator_list: L,
    ) -> RtpsReader<L, C>
    where
        C: RtpsHistoryCacheConstructor,
    {
        let spdp_builtin_participant_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);

        RtpsReader::new(
            spdp_builtin_participant_reader_guid,
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            unicast_locator_list,
            multicast_locator_list,
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        )
    }
}
