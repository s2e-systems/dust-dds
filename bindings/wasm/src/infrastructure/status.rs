use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicationMatchedStatus {
    pub(crate) inner: dust_dds::infrastructure::status::PublicationMatchedStatus,
}

#[wasm_bindgen]
impl PublicationMatchedStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }

    #[wasm_bindgen(getter, js_name = currentCount)]
    pub fn current_count(&self) -> i32 {
        self.inner.current_count
    }

    #[wasm_bindgen(getter, js_name = currentCountChange)]
    pub fn current_count_change(&self) -> i32 {
        self.inner.current_count_change
    }
}

impl From<dust_dds::infrastructure::status::PublicationMatchedStatus> for PublicationMatchedStatus {
    fn from(inner: dust_dds::infrastructure::status::PublicationMatchedStatus) -> Self {
        Self { inner }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubscriptionMatchedStatus {
    pub(crate) inner: dust_dds::infrastructure::status::SubscriptionMatchedStatus,
}

#[wasm_bindgen]
impl SubscriptionMatchedStatus {
    #[wasm_bindgen(getter, js_name = totalCount)]
    pub fn total_count(&self) -> i32 {
        self.inner.total_count
    }

    #[wasm_bindgen(getter, js_name = totalCountChange)]
    pub fn total_count_change(&self) -> i32 {
        self.inner.total_count_change
    }

    #[wasm_bindgen(getter, js_name = currentCount)]
    pub fn current_count(&self) -> i32 {
        self.inner.current_count
    }

    #[wasm_bindgen(getter, js_name = currentCountChange)]
    pub fn current_count_change(&self) -> i32 {
        self.inner.current_count_change
    }
}

impl From<dust_dds::infrastructure::status::SubscriptionMatchedStatus>
    for SubscriptionMatchedStatus
{
    fn from(inner: dust_dds::infrastructure::status::SubscriptionMatchedStatus) -> Self {
        Self { inner }
    }
}
