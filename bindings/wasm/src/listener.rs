use wasm_bindgen::prelude::*;

use crate::{
    Result,
    data_reader::DataReader,
    data_writer::DataWriter,
    dynamic_type::DynamicType,
    infrastructure::status::{
        InconsistentTopicStatus, LivelinessChangedStatus, LivelinessLostStatus,
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    result::into_js_error,
    subscriber::Subscriber,
    topic::Topic,
    type_support::JsDdsData,
};

#[derive(Clone)]
pub(crate) struct SendJsValue(pub JsValue);
unsafe impl Send for SendJsValue {}
unsafe impl Sync for SendJsValue {}

pub(crate) fn call_listener_method(listener: &JsValue, method_name: &str, args: &[JsValue]) {
    if let Ok(prop) = js_sys::Reflect::get(listener, &JsValue::from_str(method_name)) {
        if prop.is_function() {
            let func: js_sys::Function = prop.unchecked_into();
            let args_array = js_sys::Array::new();
            for arg in args {
                args_array.push(arg);
            }
            if let Err(err) = js_sys::Reflect::apply(&func, listener, &args_array) {
                web_sys::console::error_2(
                    &format!("Error executing listener method '{}':", method_name).into(),
                    &err,
                );
            }
        }
    }
}

pub(crate) fn parse_status_mask(
    mask: Option<js_sys::Array>,
) -> Result<Vec<dust_dds::infrastructure::status::StatusKind>> {
    match mask {
        Some(array) => {
            let mut list = Vec::new();
            for val in array.iter() {
                let num = val.as_f64().ok_or_else(|| {
                    into_js_error(dust_dds::infrastructure::error::DdsError::BadParameter)
                })? as u32;
                let status = match num {
                    0 => dust_dds::infrastructure::status::StatusKind::InconsistentTopic,
                    1 => dust_dds::infrastructure::status::StatusKind::OfferedDeadlineMissed,
                    2 => dust_dds::infrastructure::status::StatusKind::RequestedDeadlineMissed,
                    3 => dust_dds::infrastructure::status::StatusKind::OfferedIncompatibleQos,
                    4 => dust_dds::infrastructure::status::StatusKind::RequestedIncompatibleQos,
                    5 => dust_dds::infrastructure::status::StatusKind::SampleLost,
                    6 => dust_dds::infrastructure::status::StatusKind::SampleRejected,
                    7 => dust_dds::infrastructure::status::StatusKind::DataOnReaders,
                    8 => dust_dds::infrastructure::status::StatusKind::DataAvailable,
                    9 => dust_dds::infrastructure::status::StatusKind::LivelinessLost,
                    10 => dust_dds::infrastructure::status::StatusKind::LivelinessChanged,
                    11 => dust_dds::infrastructure::status::StatusKind::PublicationMatched,
                    12 => dust_dds::infrastructure::status::StatusKind::SubscriptionMatched,
                    _ => {
                        return Err(into_js_error(
                            dust_dds::infrastructure::error::DdsError::BadParameter,
                        ));
                    }
                };
                list.push(status);
            }
            Ok(list)
        }
        None => Ok(dust_dds::infrastructure::status::NO_STATUS.to_vec()),
    }
}

pub struct WasmDataReaderListener {
    pub(crate) listener: SendJsValue,
    pub(crate) dynamic_type: DynamicType,
}

impl dust_dds::dds_async::data_reader_listener::DataReaderListener<JsDdsData>
    for WasmDataReaderListener
{
    async fn on_data_available(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<JsDdsData>,
    ) {
        let reader = DataReader::new(the_reader, self.dynamic_type.clone());
        call_listener_method(
            &self.listener.0,
            "onDataAvailable",
            &[JsValue::from(reader)],
        );
    }

    async fn on_sample_rejected(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<JsDdsData>,
        status: dust_dds::infrastructure::status::SampleRejectedStatus,
    ) {
        let reader = DataReader::new(the_reader, self.dynamic_type.clone());
        call_listener_method(
            &self.listener.0,
            "onSampleRejected",
            &[
                JsValue::from(reader),
                JsValue::from(SampleRejectedStatus::from(status)),
            ],
        );
    }

    async fn on_liveliness_changed(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<JsDdsData>,
        status: dust_dds::infrastructure::status::LivelinessChangedStatus,
    ) {
        let reader = DataReader::new(the_reader, self.dynamic_type.clone());
        call_listener_method(
            &self.listener.0,
            "onLivelinessChanged",
            &[
                JsValue::from(reader),
                JsValue::from(LivelinessChangedStatus::from(status)),
            ],
        );
    }

    async fn on_requested_deadline_missed(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<JsDdsData>,
        status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
    ) {
        let reader = DataReader::new(the_reader, self.dynamic_type.clone());
        call_listener_method(
            &self.listener.0,
            "onRequestedDeadlineMissed",
            &[
                JsValue::from(reader),
                JsValue::from(RequestedDeadlineMissedStatus::from(status)),
            ],
        );
    }

    async fn on_requested_incompatible_qos(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<JsDdsData>,
        status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
    ) {
        let reader = DataReader::new(the_reader, self.dynamic_type.clone());
        call_listener_method(
            &self.listener.0,
            "onRequestedIncompatibleQos",
            &[
                JsValue::from(reader),
                JsValue::from(RequestedIncompatibleQosStatus::from(status)),
            ],
        );
    }

    async fn on_subscription_matched(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<JsDdsData>,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) {
        let reader = DataReader::new(the_reader, self.dynamic_type.clone());
        call_listener_method(
            &self.listener.0,
            "onSubscriptionMatched",
            &[
                JsValue::from(reader),
                JsValue::from(SubscriptionMatchedStatus::from(status)),
            ],
        );
    }

    async fn on_sample_lost(
        &mut self,
        the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<JsDdsData>,
        status: dust_dds::infrastructure::status::SampleLostStatus,
    ) {
        let reader = DataReader::new(the_reader, self.dynamic_type.clone());
        call_listener_method(
            &self.listener.0,
            "onSampleLost",
            &[
                JsValue::from(reader),
                JsValue::from(SampleLostStatus::from(status)),
            ],
        );
    }
}

pub struct WasmDataWriterListener {
    pub(crate) listener: SendJsValue,
    pub(crate) dynamic_type: DynamicType,
}

impl dust_dds::dds_async::data_writer_listener::DataWriterListener<JsDdsData>
    for WasmDataWriterListener
{
    async fn on_liveliness_lost(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<JsDdsData>,
        status: dust_dds::infrastructure::status::LivelinessLostStatus,
    ) {
        let writer = DataWriter::new(the_writer, self.dynamic_type.clone());
        call_listener_method(
            &self.listener.0,
            "onLivelinessLost",
            &[
                JsValue::from(writer),
                JsValue::from(LivelinessLostStatus::from(status)),
            ],
        );
    }

    async fn on_offered_deadline_missed(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<JsDdsData>,
        status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
    ) {
        let writer = DataWriter::new(the_writer, self.dynamic_type.clone());
        call_listener_method(
            &self.listener.0,
            "onOfferedDeadlineMissed",
            &[
                JsValue::from(writer),
                JsValue::from(OfferedDeadlineMissedStatus::from(status)),
            ],
        );
    }

    async fn on_offered_incompatible_qos(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<JsDdsData>,
        status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
    ) {
        let writer = DataWriter::new(the_writer, self.dynamic_type.clone());
        call_listener_method(
            &self.listener.0,
            "onOfferedIncompatibleQos",
            &[
                JsValue::from(writer),
                JsValue::from(OfferedIncompatibleQosStatus::from(status)),
            ],
        );
    }

    async fn on_publication_matched(
        &mut self,
        the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<JsDdsData>,
        status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) {
        let writer = DataWriter::new(the_writer, self.dynamic_type.clone());
        call_listener_method(
            &self.listener.0,
            "onPublicationMatched",
            &[
                JsValue::from(writer),
                JsValue::from(PublicationMatchedStatus::from(status)),
            ],
        );
    }
}

pub struct WasmSubscriberListener {
    pub(crate) listener: SendJsValue,
}

impl dust_dds::dds_async::subscriber_listener::SubscriberListener for WasmSubscriberListener {
    async fn on_data_on_readers(
        &mut self,
        the_subscriber: dust_dds::dds_async::subscriber::SubscriberAsync,
    ) {
        let subscriber = Subscriber::new(the_subscriber);
        call_listener_method(
            &self.listener.0,
            "onDataOnReaders",
            &[JsValue::from(subscriber)],
        );
    }

    async fn on_data_available(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
    ) {
        call_listener_method(&self.listener.0, "onDataAvailable", &[JsValue::NULL]);
    }

    async fn on_sample_rejected(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SampleRejectedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onSampleRejected",
            &[
                JsValue::NULL,
                JsValue::from(SampleRejectedStatus::from(status)),
            ],
        );
    }

    async fn on_liveliness_changed(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::LivelinessChangedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onLivelinessChanged",
            &[
                JsValue::NULL,
                JsValue::from(LivelinessChangedStatus::from(status)),
            ],
        );
    }

    async fn on_requested_deadline_missed(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onRequestedDeadlineMissed",
            &[
                JsValue::NULL,
                JsValue::from(RequestedDeadlineMissedStatus::from(status)),
            ],
        );
    }

    async fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onRequestedIncompatibleQos",
            &[
                JsValue::NULL,
                JsValue::from(RequestedIncompatibleQosStatus::from(status)),
            ],
        );
    }

    async fn on_subscription_matched(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onSubscriptionMatched",
            &[
                JsValue::NULL,
                JsValue::from(SubscriptionMatchedStatus::from(status)),
            ],
        );
    }

    async fn on_sample_lost(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SampleLostStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onSampleLost",
            &[JsValue::NULL, JsValue::from(SampleLostStatus::from(status))],
        );
    }
}

pub struct WasmPublisherListener {
    pub(crate) listener: SendJsValue,
}

impl dust_dds::dds_async::publisher_listener::PublisherListener for WasmPublisherListener {
    async fn on_liveliness_lost(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::LivelinessLostStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onLivelinessLost",
            &[
                JsValue::NULL,
                JsValue::from(LivelinessLostStatus::from(status)),
            ],
        );
    }

    async fn on_offered_deadline_missed(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onOfferedDeadlineMissed",
            &[
                JsValue::NULL,
                JsValue::from(OfferedDeadlineMissedStatus::from(status)),
            ],
        );
    }

    async fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onOfferedIncompatibleQos",
            &[
                JsValue::NULL,
                JsValue::from(OfferedIncompatibleQosStatus::from(status)),
            ],
        );
    }

    async fn on_publication_matched(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onPublicationMatched",
            &[
                JsValue::NULL,
                JsValue::from(PublicationMatchedStatus::from(status)),
            ],
        );
    }
}

pub struct WasmTopicListener {
    pub(crate) listener: SendJsValue,
}

impl dust_dds::dds_async::topic_listener::TopicListener for WasmTopicListener {
    async fn on_inconsistent_topic(
        &mut self,
        the_topic: dust_dds::dds_async::topic::TopicAsync,
        status: dust_dds::infrastructure::status::InconsistentTopicStatus,
    ) {
        let topic = Topic::new(the_topic);
        call_listener_method(
            &self.listener.0,
            "onInconsistentTopic",
            &[
                JsValue::from(topic),
                JsValue::from(InconsistentTopicStatus::from(status)),
            ],
        );
    }
}

pub struct WasmDomainParticipantListener {
    pub(crate) listener: SendJsValue,
}

impl dust_dds::dds_async::domain_participant_listener::DomainParticipantListener
    for WasmDomainParticipantListener
{
    async fn on_inconsistent_topic(
        &mut self,
        the_topic: dust_dds::dds_async::topic::TopicAsync,
        status: dust_dds::infrastructure::status::InconsistentTopicStatus,
    ) {
        let topic = Topic::new(the_topic);
        call_listener_method(
            &self.listener.0,
            "onInconsistentTopic",
            &[
                JsValue::from(topic),
                JsValue::from(InconsistentTopicStatus::from(status)),
            ],
        );
    }

    async fn on_liveliness_lost(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::LivelinessLostStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onLivelinessLost",
            &[
                JsValue::NULL,
                JsValue::from(LivelinessLostStatus::from(status)),
            ],
        );
    }

    async fn on_offered_deadline_missed(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::OfferedDeadlineMissedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onOfferedDeadlineMissed",
            &[
                JsValue::NULL,
                JsValue::from(OfferedDeadlineMissedStatus::from(status)),
            ],
        );
    }

    async fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::OfferedIncompatibleQosStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onOfferedIncompatibleQos",
            &[
                JsValue::NULL,
                JsValue::from(OfferedIncompatibleQosStatus::from(status)),
            ],
        );
    }

    async fn on_data_available(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
    ) {
        call_listener_method(&self.listener.0, "onDataAvailable", &[JsValue::NULL]);
    }

    async fn on_sample_lost(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SampleLostStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onSampleLost",
            &[JsValue::NULL, JsValue::from(SampleLostStatus::from(status))],
        );
    }

    async fn on_sample_rejected(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SampleRejectedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onSampleRejected",
            &[
                JsValue::NULL,
                JsValue::from(SampleRejectedStatus::from(status)),
            ],
        );
    }

    async fn on_liveliness_changed(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::LivelinessChangedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onLivelinessChanged",
            &[
                JsValue::NULL,
                JsValue::from(LivelinessChangedStatus::from(status)),
            ],
        );
    }

    async fn on_requested_deadline_missed(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::RequestedDeadlineMissedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onRequestedDeadlineMissed",
            &[
                JsValue::NULL,
                JsValue::from(RequestedDeadlineMissedStatus::from(status)),
            ],
        );
    }

    async fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::RequestedIncompatibleQosStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onRequestedIncompatibleQos",
            &[
                JsValue::NULL,
                JsValue::from(RequestedIncompatibleQosStatus::from(status)),
            ],
        );
    }

    async fn on_publication_matched(
        &mut self,
        _the_writer: dust_dds::dds_async::data_writer::DataWriterAsync<()>,
        status: dust_dds::infrastructure::status::PublicationMatchedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onPublicationMatched",
            &[
                JsValue::NULL,
                JsValue::from(PublicationMatchedStatus::from(status)),
            ],
        );
    }

    async fn on_subscription_matched(
        &mut self,
        _the_reader: dust_dds::dds_async::data_reader::DataReaderAsync<()>,
        status: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
    ) {
        call_listener_method(
            &self.listener.0,
            "onSubscriptionMatched",
            &[
                JsValue::NULL,
                JsValue::from(SubscriptionMatchedStatus::from(status)),
            ],
        );
    }
}
