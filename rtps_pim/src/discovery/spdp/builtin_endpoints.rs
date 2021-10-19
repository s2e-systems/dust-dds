use crate::{
    behavior::{
        reader::{reader::RtpsReader, stateless_reader::RtpsStatelessReader},
        types::DURATION_ZERO,
        writer::{stateless_writer::RtpsStatelessWriter, writer::RtpsWriter},
    },
    structure::{
        types::{
            EntityId, Guid, GuidPrefix, ReliabilityKind, TopicKind, BUILT_IN_READER_WITH_KEY,
            BUILT_IN_WRITER_WITH_KEY,
        },
        RtpsEndpoint, RtpsHistoryCache,
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
        reader_locators: R,
    ) -> RtpsStatelessWriter<L, C, R>
    where
        C: for<'a> RtpsHistoryCache<'a>,
    {
        let spdp_builtin_participant_writer_guid =
            Guid::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_WRITER);

        RtpsStatelessWriter {
            writer: RtpsWriter {
                endpoint: RtpsEndpoint::new(
                    spdp_builtin_participant_writer_guid,
                    TopicKind::WithKey,
                    ReliabilityKind::BestEffort,
                    unicast_locator_list,
                    multicast_locator_list,
                ),
                push_mode: true,
                heartbeat_period: DURATION_ZERO,
                nack_response_delay: DURATION_ZERO,
                nack_suppression_duration: DURATION_ZERO,
                last_change_sequence_number: 0,
                data_max_size_serialized: None,
                writer_cache: C::new(),
            },
            reader_locators,
        }
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

        RtpsStatelessReader(RtpsReader::new(
            spdp_builtin_participant_reader_guid,
            TopicKind::WithKey,
            ReliabilityKind::BestEffort,
            unicast_locator_list,
            multicast_locator_list,
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        ))
    }
}
