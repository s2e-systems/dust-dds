use crate::DustDdsStatusCondition;
use crate::infrastructure::condition::DustDdsStatusMask;
use crate::infrastructure::error::{RETCODE_BAD_PARAMETER, RETCODE_OK, ReturnCode};
use crate::infrastructure::listeners::{CDataReaderListenerWrapper, DustDdsDataReaderListener};
use crate::infrastructure::qos::DataReaderQos;
use crate::subscription::subscriber::DustDdsSubscriber;
use crate::topic_definition::topic::DustDdsTopic;
use dust_dds::xtypes::dynamic_type::DynamicData;
use std::ptr::NonNull;

/// cbindgen:opaque
pub struct DustDdsDataReader(
    pub(crate) dust_dds::subscription::data_reader::DataReader<DynamicData<'static>>,
);

impl DustDdsDataReader {
    pub fn new(
        data_reader: dust_dds::subscription::data_reader::DataReader<DynamicData<'static>>,
    ) -> Self {
        Self(data_reader)
    }

    pub fn inner(&self) -> &dust_dds::subscription::data_reader::DataReader<DynamicData<'static>> {
        &self.0
    }
}

/// Creates a new DataReader.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
/// - `topic` must point to a valid, initialized `DustDdsTopic` instance.
/// - `qos` must be a valid pointer to a `DataReaderQos` instance (or null).
/// - `listener` must be a valid pointer to a `DustDdsDataReaderListener` instance (or null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_subscriber_create_datareader(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    topic: Option<NonNull<DustDdsTopic>>,
    qos: *const DataReaderQos,
    listener: *const DustDdsDataReaderListener,
    mask: DustDdsStatusMask,
) -> Option<NonNull<DustDdsDataReader>> {
    let subscriber = subscriber?;
    let topic = topic?;

    let qos = if qos.is_null() {
        dust_dds::infrastructure::qos::QosKind::Default
    } else {
        dust_dds::infrastructure::qos::QosKind::Specific((*unsafe { &*qos }).into())
    };

    let status_kinds = crate::infrastructure::condition::mask_to_status_kinds(mask);

    let subscriber_ref = unsafe { subscriber.as_ref() };
    let topic_ref = unsafe { topic.as_ref() };

    if listener.is_null() {
        struct NoDataReaderListener;
        impl dust_dds::subscription::data_reader_listener::DataReaderListener<DynamicData<'static>>
            for NoDataReaderListener
        {
        }

        match subscriber_ref
            .inner()
            .create_datareader::<DynamicData<'static>>(
                topic_ref.inner(),
                qos,
                None::<NoDataReaderListener>,
                &status_kinds,
            ) {
            Ok(dr) => NonNull::new(Box::into_raw(Box::new(DustDdsDataReader::new(dr)))),
            Err(_) => None,
        }
    } else {
        let listener_wrapper = CDataReaderListenerWrapper {
            listener: unsafe { *listener },
        };

        match subscriber_ref
            .inner()
            .create_datareader::<DynamicData<'static>>(
                topic_ref.inner(),
                qos,
                Some(listener_wrapper),
                &status_kinds,
            ) {
            Ok(dr) => NonNull::new(Box::into_raw(Box::new(DustDdsDataReader::new(dr)))),
            Err(_) => None,
        }
    }
}

/// Deletes an existing DataReader.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `subscriber` must point to a valid, initialized `DustDdsSubscriber` instance.
/// - `datareader` must point to a valid, initialized `DustDdsDataReader` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_subscriber_delete_datareader(
    subscriber: Option<NonNull<DustDdsSubscriber>>,
    datareader: Option<NonNull<DustDdsDataReader>>,
) -> ReturnCode {
    let Some(datareader) = datareader else {
        return RETCODE_OK;
    };
    let Some(subscriber) = subscriber else {
        return RETCODE_BAD_PARAMETER;
    };

    let subscriber_ref = unsafe { subscriber.as_ref() };
    let datareader_ref = unsafe { datareader.as_ref() };

    match subscriber_ref
        .inner()
        .delete_datareader(datareader_ref.inner())
    {
        Ok(()) => {
            unsafe {
                drop(Box::from_raw(datareader.as_ptr()));
            }
            RETCODE_OK
        }
        Err(e) => e.into(),
    }
}

/// Gets the StatusCondition associated with the DataReader.
///
/// # Safety
///
/// The caller must observe the following safety invariants:
/// - `reader` must point to a valid, initialized `DustDdsDataReader` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dds_datareader_get_statuscondition(
    reader: Option<NonNull<DustDdsDataReader>>,
) -> Option<NonNull<DustDdsStatusCondition>> {
    let reader = reader?;

    let reader_ref = unsafe { reader.as_ref() };
    let condition = reader_ref.inner().get_statuscondition();
    NonNull::new(Box::into_raw(Box::new(DustDdsStatusCondition::new(
        condition,
    ))))
}

pub type SampleStateMask = u32;
pub type ViewStateMask = u32;
pub type InstanceStateMask = u32;

pub const READ_SAMPLE_STATE: SampleStateMask = 0x0001;
pub const NOT_READ_SAMPLE_STATE: SampleStateMask = 0x0002;
pub const ANY_SAMPLE_STATE: SampleStateMask = 0xffff;

pub const NEW_VIEW_STATE: ViewStateMask = 0x0001;
pub const NOT_NEW_VIEW_STATE: ViewStateMask = 0x0002;
pub const ANY_VIEW_STATE: ViewStateMask = 0xffff;

pub const ALIVE_INSTANCE_STATE: InstanceStateMask = 0x0001;
pub const NOT_ALIVE_DISPOSED_INSTANCE_STATE: InstanceStateMask = 0x0002;
pub const NOT_ALIVE_NO_WRITERS_INSTANCE_STATE: InstanceStateMask = 0x0004;
pub const ANY_INSTANCE_STATE: InstanceStateMask = 0xffff;
pub const NOT_ALIVE_INSTANCE_STATE: InstanceStateMask = 0x0006;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SampleInfo {
    pub sample_state: SampleStateMask,
    pub view_state: ViewStateMask,
    pub instance_state: InstanceStateMask,
    pub disposed_generation_count: i32,
    pub no_writers_generation_count: i32,
    pub sample_rank: i32,
    pub generation_rank: i32,
    pub absolute_generation_rank: i32,
    pub source_timestamp: crate::infrastructure::qos_policy::Time_t,
    pub instance_handle: crate::infrastructure::status::InstanceHandle_t,
    pub publication_handle: crate::infrastructure::status::InstanceHandle_t,
    pub valid_data: bool,
}

pub(crate) fn sample_state_kind_to_mask(
    value: dust_dds::infrastructure::sample_info::SampleStateKind,
) -> SampleStateMask {
    match value {
        dust_dds::infrastructure::sample_info::SampleStateKind::Read => READ_SAMPLE_STATE,
        dust_dds::infrastructure::sample_info::SampleStateKind::NotRead => NOT_READ_SAMPLE_STATE,
    }
}

pub(crate) fn view_state_kind_to_mask(
    value: dust_dds::infrastructure::sample_info::ViewStateKind,
) -> ViewStateMask {
    match value {
        dust_dds::infrastructure::sample_info::ViewStateKind::New => NEW_VIEW_STATE,
        dust_dds::infrastructure::sample_info::ViewStateKind::NotNew => NOT_NEW_VIEW_STATE,
    }
}

pub(crate) fn instance_state_kind_to_mask(
    value: dust_dds::infrastructure::sample_info::InstanceStateKind,
) -> InstanceStateMask {
    match value {
        dust_dds::infrastructure::sample_info::InstanceStateKind::Alive => ALIVE_INSTANCE_STATE,
        dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveDisposed => {
            NOT_ALIVE_DISPOSED_INSTANCE_STATE
        }
        dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveNoWriters => {
            NOT_ALIVE_NO_WRITERS_INSTANCE_STATE
        }
    }
}

impl From<dust_dds::infrastructure::sample_info::SampleInfo> for SampleInfo {
    fn from(value: dust_dds::infrastructure::sample_info::SampleInfo) -> Self {
        Self {
            sample_state: sample_state_kind_to_mask(value.sample_state),
            view_state: view_state_kind_to_mask(value.view_state),
            instance_state: instance_state_kind_to_mask(value.instance_state),
            disposed_generation_count: value.disposed_generation_count,
            no_writers_generation_count: value.no_writers_generation_count,
            sample_rank: value.sample_rank,
            generation_rank: value.generation_rank,
            absolute_generation_rank: value.absolute_generation_rank,
            source_timestamp: value.source_timestamp.into(),
            instance_handle: value.instance_handle.into(),
            publication_handle: value.publication_handle.into(),
            valid_data: value.valid_data,
        }
    }
}

pub(crate) fn sample_states_from_mask(
    mask: SampleStateMask,
) -> Vec<dust_dds::infrastructure::sample_info::SampleStateKind> {
    let mut states = Vec::new();
    if (mask & READ_SAMPLE_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::SampleStateKind::Read);
    }
    if (mask & NOT_READ_SAMPLE_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::SampleStateKind::NotRead);
    }
    states
}

pub(crate) fn view_states_from_mask(
    mask: ViewStateMask,
) -> Vec<dust_dds::infrastructure::sample_info::ViewStateKind> {
    let mut states = Vec::new();
    if (mask & NEW_VIEW_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::ViewStateKind::New);
    }
    if (mask & NOT_NEW_VIEW_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::ViewStateKind::NotNew);
    }
    states
}

pub(crate) fn instance_states_from_mask(
    mask: InstanceStateMask,
) -> Vec<dust_dds::infrastructure::sample_info::InstanceStateKind> {
    let mut states = Vec::new();
    if (mask & ALIVE_INSTANCE_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::InstanceStateKind::Alive);
    }
    if (mask & NOT_ALIVE_DISPOSED_INSTANCE_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveDisposed);
    }
    if (mask & NOT_ALIVE_NO_WRITERS_INSTANCE_STATE) != 0 {
        states.push(dust_dds::infrastructure::sample_info::InstanceStateKind::NotAliveNoWriters);
    }
    states
}
