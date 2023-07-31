use crate::{
    implementation::{rtps_udp_psm::udp_transport::UdpTransportWrite, utils::actor::ActorAddress},
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::DataReaderQos,
        qos_policy::ReliabilityQosPolicyKind,
        status::SampleRejectedStatusKind,
        time::{Duration, Time, DURATION_ZERO},
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    messages::{
        overall_structure::RtpsMessageHeader,
        submessages::{
            data::DataSubmessageRead, data_frag::DataFragSubmessageRead, gap::GapSubmessageRead,
            heartbeat::HeartbeatSubmessageRead, heartbeat_frag::HeartbeatFragSubmessageRead,
        },
    },
    reader::{convert_data_frag_to_cache_change, RtpsReader, RtpsReaderError},
    types::{Guid, GuidPrefix, Locator, SequenceNumber},
    writer_proxy::RtpsWriterProxy,
};

pub const DEFAULT_HEARTBEAT_RESPONSE_DELAY: Duration = Duration::new(0, 500);
pub const DEFAULT_HEARTBEAT_SUPPRESSION_DURATION: Duration = DURATION_ZERO;

/// ChangeFromWriterStatusKind
/// Enumeration used to indicate the status of a ChangeFromWriter. It can take the values:
/// LOST, MISSING, RECEIVED, UNKNOWN
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChangeFromWriterStatusKind {
    Lost,
    Missing,
    Received,
    Unknown,
}

pub enum StatefulReaderDataReceivedResult {
    NoMatchedWriterProxy,
    UnexpectedDataSequenceNumber,
    NewSampleAdded(InstanceHandle),
    NewSampleAddedAndSamplesLost(InstanceHandle),
    SampleRejected(InstanceHandle, SampleRejectedStatusKind),
    InvalidData(&'static str),
}

impl From<RtpsReaderError> for StatefulReaderDataReceivedResult {
    fn from(e: RtpsReaderError) -> Self {
        match e {
            RtpsReaderError::InvalidData(s) => StatefulReaderDataReceivedResult::InvalidData(s),
            RtpsReaderError::Rejected(instance_handle, reason) => {
                StatefulReaderDataReceivedResult::SampleRejected(instance_handle, reason)
            }
        }
    }
}

pub struct RtpsStatefulReader {
    reader: RtpsReader,
    matched_writers: Vec<RtpsWriterProxy>,
}

impl RtpsStatefulReader {
    pub fn new(reader: RtpsReader) -> Self {
        Self {
            reader,
            matched_writers: Vec::new(),
        }
    }

    pub fn guid(&self) -> Guid {
        self.reader.guid()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.reader.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.reader.multicast_locator_list()
    }

    pub fn get_qos(&self) -> &DataReaderQos {
        self.reader.get_qos()
    }

    pub fn set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
        self.reader.set_qos(qos)
    }

    pub fn read<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.reader.read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn take<Foo>(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.reader.take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn read_next_instance<Foo>(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.reader.read_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn take_next_instance<Foo>(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.reader.take_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy) {
        if !self
            .matched_writers
            .iter()
            .any(|x| x.remote_writer_guid() == a_writer_proxy.remote_writer_guid())
        {
            self.matched_writers.push(a_writer_proxy);
        }
    }

    pub fn matched_writer_remove(&mut self, a_writer_guid: Guid) {
        self.matched_writers
            .retain(|x| x.remote_writer_guid() != a_writer_guid)
    }

    pub fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessageRead<'_>,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<Time>,
        reception_timestamp: Time,
    ) -> StatefulReaderDataReceivedResult {
        let sequence_number = data_submessage.writer_sn();
        let writer_guid = Guid::new(source_guid_prefix, data_submessage.writer_id());

        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|wp| wp.remote_writer_guid() == writer_guid)
        {
            let expected_seq_num = writer_proxy.available_changes_max() + 1;
            match self.reader.get_qos().reliability.kind {
                ReliabilityQosPolicyKind::BestEffort => {
                    if sequence_number >= expected_seq_num {
                        let change_result = self.reader.convert_data_to_cache_change(
                            data_submessage,
                            source_timestamp,
                            source_guid_prefix,
                            reception_timestamp,
                        );
                        match change_result {
                            Ok(change) => {
                                let add_change_result = self.reader.add_change(change);

                                match add_change_result {
                                    Ok(instance_handle) => {
                                        writer_proxy.received_change_set(sequence_number);
                                        if sequence_number > expected_seq_num {
                                            writer_proxy.lost_changes_update(sequence_number);
                                            StatefulReaderDataReceivedResult::NewSampleAddedAndSamplesLost(instance_handle)
                                        } else {
                                            StatefulReaderDataReceivedResult::NewSampleAdded(
                                                instance_handle,
                                            )
                                        }
                                    }
                                    Err(err) => err.into(),
                                }
                            }
                            Err(_) => StatefulReaderDataReceivedResult::InvalidData(
                                "Invalid data submessage",
                            ),
                        }
                    } else {
                        StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
                ReliabilityQosPolicyKind::Reliable => {
                    if sequence_number == expected_seq_num {
                        let change_result = self.reader.convert_data_to_cache_change(
                            data_submessage,
                            source_timestamp,
                            source_guid_prefix,
                            reception_timestamp,
                        );
                        match change_result {
                            Ok(change) => {
                                let add_change_result = self.reader.add_change(change);

                                match add_change_result {
                                    Ok(instance_handle) => {
                                        writer_proxy
                                            .received_change_set(data_submessage.writer_sn());
                                        StatefulReaderDataReceivedResult::NewSampleAdded(
                                            instance_handle,
                                        )
                                    }
                                    Err(err) => err.into(),
                                }
                            }
                            Err(_) => StatefulReaderDataReceivedResult::InvalidData(
                                "Invalid data submessage",
                            ),
                        }
                    } else {
                        StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
            }
        } else {
            StatefulReaderDataReceivedResult::NoMatchedWriterProxy
        }
    }

    pub fn on_data_frag_submessage_received(
        &mut self,
        data_frag_submessage: &DataFragSubmessageRead<'_>,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<Time>,
        reception_timestamp: Time,
    ) -> StatefulReaderDataReceivedResult {
        let sequence_number = data_frag_submessage.writer_sn();
        let writer_guid = Guid::new(source_guid_prefix, data_frag_submessage.writer_id());

        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|wp| wp.remote_writer_guid() == writer_guid)
        {
            let expected_seq_num = writer_proxy.available_changes_max() + 1;
            match self.reader.get_qos().reliability.kind {
                ReliabilityQosPolicyKind::BestEffort => {
                    if sequence_number >= expected_seq_num {
                        writer_proxy.push_data_frag(data_frag_submessage);
                        if let Some(data) = writer_proxy.extract_frag(sequence_number) {
                            let change_results = convert_data_frag_to_cache_change(
                                data_frag_submessage,
                                data,
                                source_timestamp,
                                source_guid_prefix,
                                reception_timestamp,
                            );
                            match change_results {
                                Ok(change) => {
                                    let add_change_result = self.reader.add_change(change);
                                    match add_change_result {
                                        Ok(instance_handle) => {
                                            writer_proxy.received_change_set(sequence_number);
                                            if sequence_number > expected_seq_num {
                                                writer_proxy.lost_changes_update(sequence_number);
                                                StatefulReaderDataReceivedResult::NewSampleAddedAndSamplesLost(instance_handle)
                                            } else {
                                                StatefulReaderDataReceivedResult::NewSampleAdded(
                                                    instance_handle,
                                                )
                                            }
                                        }
                                        Err(err) => err.into(),
                                    }
                                }
                                Err(_) => StatefulReaderDataReceivedResult::InvalidData(
                                    "Invalid data submessage",
                                ),
                            }
                        } else {
                            StatefulReaderDataReceivedResult::NoMatchedWriterProxy
                        }
                    } else {
                        StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
                ReliabilityQosPolicyKind::Reliable => {
                    if sequence_number == expected_seq_num {
                        writer_proxy.push_data_frag(data_frag_submessage);
                        if let Some(data) = writer_proxy.extract_frag(sequence_number) {
                            let change_result = convert_data_frag_to_cache_change(
                                data_frag_submessage,
                                data,
                                source_timestamp,
                                source_guid_prefix,
                                reception_timestamp,
                            );

                            match change_result {
                                Ok(change) => {
                                    let add_change_result = self.reader.add_change(change);
                                    match add_change_result {
                                        Ok(instance_handle) => {
                                            writer_proxy.received_change_set(sequence_number);
                                            StatefulReaderDataReceivedResult::NewSampleAdded(
                                                instance_handle,
                                            )
                                        }
                                        Err(err) => err.into(),
                                    }
                                }
                                Err(_) => StatefulReaderDataReceivedResult::InvalidData(
                                    "Invalid data submessage",
                                ),
                            }
                        } else {
                            StatefulReaderDataReceivedResult::NoMatchedWriterProxy
                        }
                    } else {
                        StatefulReaderDataReceivedResult::UnexpectedDataSequenceNumber
                    }
                }
            }
        } else {
            StatefulReaderDataReceivedResult::NoMatchedWriterProxy
        }
    }

    pub fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.reader.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let writer_guid = Guid::new(source_guid_prefix, heartbeat_submessage.writer_id());

            if let Some(writer_proxy) = self
                .matched_writers
                .iter_mut()
                .find(|x| x.remote_writer_guid() == writer_guid)
            {
                if writer_proxy.last_received_heartbeat_count() < heartbeat_submessage.count() {
                    writer_proxy.set_last_received_heartbeat_count(heartbeat_submessage.count());

                    writer_proxy.set_must_send_acknacks(
                        !heartbeat_submessage.final_flag()
                            || (!heartbeat_submessage.liveliness_flag()
                                && !writer_proxy.missing_changes().is_empty()),
                    );

                    if !heartbeat_submessage.final_flag() {
                        writer_proxy.set_must_send_acknacks(true);
                    }
                    writer_proxy.missing_changes_update(heartbeat_submessage.last_sn());
                    writer_proxy.lost_changes_update(heartbeat_submessage.first_sn());
                }
            }
        }
    }

    pub fn on_heartbeat_frag_submessage_received(
        &mut self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.reader.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let writer_guid = Guid::new(source_guid_prefix, heartbeat_frag_submessage.writer_id());

            if let Some(writer_proxy) = self
                .matched_writers
                .iter_mut()
                .find(|x| x.remote_writer_guid() == writer_guid)
            {
                if writer_proxy.last_received_heartbeat_count() < heartbeat_frag_submessage.count()
                {
                    writer_proxy
                        .set_last_received_heartbeat_frag_count(heartbeat_frag_submessage.count());
                }

                // todo!()
            }
        }
    }

    pub fn on_gap_submessage_received(
        &mut self,
        gap_submessage: &GapSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, gap_submessage.writer_id());
        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == writer_guid)
        {
            for seq_num in
                i64::from(gap_submessage.gap_start())..i64::from(gap_submessage.gap_list().base())
            {
                writer_proxy.irrelevant_change_set(SequenceNumber::new(seq_num))
            }

            for seq_num in gap_submessage.gap_list().set() {
                writer_proxy.irrelevant_change_set(*seq_num)
            }
        }
    }

    pub fn send_message(
        &mut self,
        header: RtpsMessageHeader,
        udp_transport_write: &ActorAddress<UdpTransportWrite>,
    ) {
        for writer_proxy in self.matched_writers.iter_mut() {
            writer_proxy.send_message(&self.reader.guid(), header, udp_transport_write)
        }
    }

    pub fn is_historical_data_received(&self) -> bool {
        !self
            .matched_writers
            .iter()
            .any(|p| !p.is_historical_data_received())
    }
}
