use crate::{
    behavior::{
        reader::stateless_reader::RtpsStatelessReader, types::DURATION_ZERO,
        writer::stateless_writer::RtpsStatelessWriter,
    },
    structure::{
        types::{
            EntityId, Guid, GuidPrefix, ReliabilityKind, TopicKind, BUILT_IN_READER_WITH_KEY,
            BUILT_IN_WRITER_WITH_KEY,
        },
        RtpsHistoryCache,
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
        C: for<'a> RtpsHistoryCache<'a>,
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
    ) -> RtpsStatelessReader<L, C>
    where
        C: for<'a> RtpsHistoryCache<'a>,
    {
        let spdp_builtin_participant_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);

        RtpsStatelessReader::new(
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
