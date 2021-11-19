use rust_rtps_pim::{
    behavior::{types::DURATION_ZERO, writer::writer::RtpsWriter},
    structure::{
        history_cache::RtpsHistoryCacheConstructor,
        types::{
            EntityId, Guid, GuidPrefix, Locator, ReliabilityKind, TopicKind,
            BUILT_IN_READER_WITH_KEY, BUILT_IN_WRITER_WITH_KEY,
        },
    },
};

use crate::rtps_stateless_reader_impl::RtpsStatelessReaderImpl;

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_WRITER_WITH_KEY);

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER: EntityId =
    EntityId::new([0x00, 0x01, 0x00], BUILT_IN_READER_WITH_KEY);

pub struct SpdpBuiltinParticipantWriter;

impl SpdpBuiltinParticipantWriter {
    pub fn create<L, C>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: L,
        multicast_locator_list: L,
    ) -> RtpsWriter<L, C>
    where
        C: RtpsHistoryCacheConstructor,
    {
        let spdp_builtin_participant_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);

        RtpsWriter::new(
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
    pub fn create<C>(
        guid_prefix: GuidPrefix,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
    ) -> RtpsStatelessReaderImpl<C>
    where
        C: RtpsHistoryCacheConstructor,
    {
        let spdp_builtin_participant_reader_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER);

        RtpsStatelessReaderImpl::new(
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
