use std::sync::{atomic, Arc, Mutex, Weak};

use rust_dds_api::{
    dcps_psm::StatusMask,
    dds_type::DDSType,
    infrastructure::qos::{DataReaderQos, SubscriberQos},
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader_listener::DataReaderListener, subscriber_listener::SubscriberListener,
    },
};
use rust_rtps::{
    structure::{RTPSEntity, RTPSGroup},
    types::GUID,
};

use super::{data_reader_impl::DataReaderImpl, topic_impl::TopicImpl};

pub struct SubscriberImpl {
    reader_list: Vec<Arc<Mutex<DataReaderImpl>>>,
    reader_count: atomic::AtomicU8,
    default_datareader_qos: DataReaderQos,
    qos: SubscriberQos,
    listener: Option<Box<dyn SubscriberListener>>,
    status_mask: StatusMask,
}

impl SubscriberImpl {
    pub fn new(
        qos: SubscriberQos,
        listener: Option<Box<dyn SubscriberListener>>,
        status_mask: StatusMask,
    ) -> Self {
        Self {
            reader_list: Default::default(),
            reader_count: atomic::AtomicU8::new(0),
            default_datareader_qos: DataReaderQos::default(),
            qos,
            listener,
            status_mask,
        }
    }

    pub fn reader_list(&self) -> &Vec<Arc<Mutex<DataReaderImpl>>> {
        &self.reader_list
    }

    pub fn create_datareader<'a, T: DDSType>(
        &'a mut self,
        topic: Arc<Mutex<TopicImpl>>,
        qos: Option<DataReaderQos>,
        a_listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        mask: StatusMask,
    ) -> Option<Weak<Mutex<DataReaderImpl>>> {
        let qos = qos.unwrap_or(self.default_datareader_qos.clone());
        qos.is_consistent().ok()?;

        todo!()
        // let data_reader = Arc::new(Mutex::new(DataReaderImpl::new(
        //     topic, qos, a_listener, mask,
        // )));

        // self.reader_list.push(data_reader.clone());

        // Some(Arc::downgrade(&data_reader))
    }

    pub fn delete_datareader(
        &mut self,
        a_datareader: &Weak<Mutex<DataReaderImpl>>,
    ) -> DDSResult<()> {
        let datareader_impl = a_datareader.upgrade().ok_or(DDSError::AlreadyDeleted)?;
        self.reader_list
            .retain(|x| !std::ptr::eq(x.as_ref(), datareader_impl.as_ref()));
        Ok(())
    }

    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.clone()
    }

    pub fn set_qos(&mut self, qos: Option<SubscriberQos>) {
        let qos = qos.unwrap_or_default();
        self.qos = qos;
    }
}

impl RTPSEntity for SubscriberImpl {
    fn guid(&self) -> GUID {
        todo!()
    }
}

impl RTPSGroup for SubscriberImpl {}
