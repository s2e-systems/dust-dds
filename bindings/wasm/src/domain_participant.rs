use dust_dds::infrastructure::{listener::NO_LISTENER, qos::QosKind, status::NO_STATUS};
use wasm_bindgen::prelude::*;

use crate::{
    infrastructure::{
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, SubscriberQos, TopicQos},
    },
    publisher::Publisher,
    result::{Result, into_js_error},
    subscriber::Subscriber,
    topic::Topic,
};

#[wasm_bindgen]
#[derive(Clone)]
pub struct DomainParticipant {
    pub(crate) _inner: dust_dds::dds_async::domain_participant::DomainParticipantAsync,
}

impl DomainParticipant {
    pub(crate) fn new(
        inner: dust_dds::dds_async::domain_participant::DomainParticipantAsync,
    ) -> Self {
        Self { _inner: inner }
    }
}

#[wasm_bindgen]
impl DomainParticipant {
    #[wasm_bindgen(js_name = getDomainId)]
    pub fn get_domain_id(&self) -> i32 {
        self._inner.get_domain_id()
    }

    #[wasm_bindgen(js_name = getInstanceHandle)]
    pub fn get_instance_handle(&self) -> InstanceHandle {
        self._inner.get_instance_handle().into()
    }

    pub async fn enable(&self) -> Result<()> {
        self._inner.enable().await.map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = deleteContainedEntities)]
    pub async fn delete_contained_entities(&self) -> Result<()> {
        self._inner
            .delete_contained_entities()
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getQos)]
    pub async fn get_qos(&self) -> Result<DomainParticipantQos> {
        let qos = self._inner.get_qos().await.map_err(into_js_error)?;
        Ok(qos.into())
    }

    #[wasm_bindgen(js_name = setQos)]
    pub async fn set_qos(&self, qos: Option<DomainParticipantQos>) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self._inner.set_qos(qos).await.map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = createPublisher)]
    pub async fn create_publisher(&self, qos: Option<PublisherQos>) -> Result<Publisher> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        let publisher = self
            ._inner
            .create_publisher(qos, NO_LISTENER, NO_STATUS)
            .await
            .map_err(into_js_error)?;
        Ok(Publisher::new(publisher))
    }

    #[wasm_bindgen(js_name = deletePublisher)]
    pub async fn delete_publisher(&self, a_publisher: &Publisher) -> Result<()> {
        self._inner
            .delete_publisher(&a_publisher._inner)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getDefaultPublisherQos)]
    pub async fn get_default_publisher_qos(&self) -> Result<PublisherQos> {
        let qos = self
            ._inner
            .get_default_publisher_qos()
            .await
            .map_err(into_js_error)?;
        Ok(qos.into())
    }

    #[wasm_bindgen(js_name = setDefaultPublisherQos)]
    pub async fn set_default_publisher_qos(&self, qos: Option<PublisherQos>) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self._inner
            .set_default_publisher_qos(qos)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = createSubscriber)]
    pub async fn create_subscriber(&self, qos: Option<SubscriberQos>) -> Result<Subscriber> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        let subscriber = self
            ._inner
            .create_subscriber(qos, NO_LISTENER, NO_STATUS)
            .await
            .map_err(into_js_error)?;
        Ok(Subscriber::new(subscriber))
    }

    #[wasm_bindgen(js_name = deleteSubscriber)]
    pub async fn delete_subscriber(&self, a_subscriber: &Subscriber) -> Result<()> {
        self._inner
            .delete_subscriber(&a_subscriber._inner)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getBuiltinSubscriber)]
    pub fn get_builtin_subscriber(&self) -> Subscriber {
        Subscriber::new(self._inner.get_builtin_subscriber())
    }

    #[wasm_bindgen(js_name = getDefaultSubscriberQos)]
    pub async fn get_default_subscriber_qos(&self) -> Result<SubscriberQos> {
        let qos = self
            ._inner
            .get_default_subscriber_qos()
            .await
            .map_err(into_js_error)?;
        Ok(qos.into())
    }

    #[wasm_bindgen(js_name = setDefaultSubscriberQos)]
    pub async fn set_default_subscriber_qos(&self, qos: Option<SubscriberQos>) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self._inner
            .set_default_subscriber_qos(qos)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = createTopic)]
    pub async fn create_topic(
        &self,
        topic_name: &str,
        dynamic_type: &crate::dynamic_type::DynamicType,
        qos: Option<TopicQos>,
    ) -> Result<Topic> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };

        let type_name = dynamic_type.inner.get_name();

        let topic = self
            ._inner
            .create_dynamic_topic(
                topic_name,
                type_name,
                qos,
                NO_LISTENER,
                NO_STATUS,
                dynamic_type.inner,
            )
            .await
            .map_err(into_js_error)?;
        Ok(Topic::new(topic))
    }

    #[wasm_bindgen(js_name = deleteTopic)]
    pub async fn delete_topic(&self, a_topic: &Topic) -> Result<()> {
        self._inner
            .delete_topic(&a_topic._inner)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getDefaultTopicQos)]
    pub async fn get_default_topic_qos(&self) -> Result<TopicQos> {
        let qos = self
            ._inner
            .get_default_topic_qos()
            .await
            .map_err(into_js_error)?;
        Ok(qos.into())
    }

    #[wasm_bindgen(js_name = setDefaultTopicQos)]
    pub async fn set_default_topic_qos(&self, qos: Option<TopicQos>) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self._inner
            .set_default_topic_qos(qos)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = ignoreParticipant)]
    pub async fn ignore_participant(&self, handle: &InstanceHandle) -> Result<()> {
        self._inner
            .ignore_participant(handle.into())
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = ignorePublication)]
    pub async fn ignore_publication(&self, handle: &InstanceHandle) -> Result<()> {
        self._inner
            .ignore_publication(handle.into())
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = ignoreSubscription)]
    pub async fn ignore_subscription(&self, handle: &InstanceHandle) -> Result<()> {
        self._inner
            .ignore_subscription(handle.into())
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getDiscoveredParticipants)]
    pub async fn get_discovered_participants(&self) -> Result<js_sys::Array> {
        let participants = self
            ._inner
            .get_discovered_participants()
            .await
            .map_err(into_js_error)?;
        let array = js_sys::Array::new();
        for h in participants {
            array.push(&JsValue::from(InstanceHandle::from(h)));
        }
        Ok(array)
    }

    #[wasm_bindgen(js_name = getDiscoveredTopics)]
    pub async fn get_discovered_topics(&self) -> Result<js_sys::Array> {
        let topics = self
            ._inner
            .get_discovered_topics()
            .await
            .map_err(into_js_error)?;
        let array = js_sys::Array::new();
        for h in topics {
            array.push(&JsValue::from(InstanceHandle::from(h)));
        }
        Ok(array)
    }
}
