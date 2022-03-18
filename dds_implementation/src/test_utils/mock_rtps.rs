use crate::utils::rtps_structure::RtpsStructure;

use super::{
    mock_rtps_cache_change::MockRtpsCacheChange, mock_rtps_group::MockRtpsGroup,
    mock_rtps_history_cache::MockRtpsHistoryCache, mock_rtps_participant::MockRtpsParticipant,
    mock_rtps_stateful_reader::MockRtpsStatefulReader,
    mock_rtps_stateful_writer::MockRtpsStatefulWriter,
    mock_rtps_stateless_reader::MockRtpsStatelessReader,
    mock_rtps_stateless_writer::MockRtpsStatelessWriter,
};

pub struct MockRtps;

impl RtpsStructure for MockRtps {
    type Group = MockRtpsGroup;
    type Participant = MockRtpsParticipant;
    type StatelessWriter = MockRtpsStatelessWriter;
    type StatefulWriter = MockRtpsStatefulWriter;
    type StatelessReader = MockRtpsStatelessReader;
    type StatefulReader = MockRtpsStatefulReader;
    type HistoryCache = MockRtpsHistoryCache;
    type CacheChange = MockRtpsCacheChange;
}
