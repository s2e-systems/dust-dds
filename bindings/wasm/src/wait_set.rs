use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

use crate::{
    Result,
    condition::{Condition, StatusCondition},
    result::into_js_error,
};

#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct WaitSet {
    conditions: Rc<RefCell<Vec<Condition>>>,
}

#[wasm_bindgen]
impl WaitSet {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            conditions: Rc::new(RefCell::new(Vec::new())),
        }
    }

    #[wasm_bindgen(js_name = attachCondition)]
    pub fn attach_condition(&self, cond: &Condition) -> Result<()> {
        let mut conditions = self.conditions.borrow_mut();
        if !conditions
            .iter()
            .any(|c| c.entity_handle == cond.entity_handle)
        {
            conditions.push(cond.clone());
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = attachStatusCondition)]
    pub fn attach_status_condition(&self, cond: &StatusCondition) -> Result<()> {
        self.attach_condition(&cond.as_condition())
    }

    #[wasm_bindgen(js_name = detachCondition)]
    pub fn detach_condition(&self, cond: &Condition) -> Result<()> {
        let mut conditions = self.conditions.borrow_mut();
        let index = conditions
            .iter()
            .position(|c| c.entity_handle == cond.entity_handle)
            .ok_or_else(|| {
                into_js_error(
                    dust_dds::infrastructure::error::DdsError::PreconditionNotMet(String::from(
                        "Condition not attached to WaitSet",
                    )),
                )
            })?;
        conditions.remove(index);
        Ok(())
    }

    #[wasm_bindgen(js_name = detachStatusCondition)]
    pub fn detach_status_condition(&self, cond: &StatusCondition) -> Result<()> {
        self.detach_condition(&cond.as_condition())
    }

    #[wasm_bindgen(js_name = getConditions)]
    pub fn get_conditions(&self) -> js_sys::Array {
        let conditions = self.conditions.borrow();
        let array = js_sys::Array::new();
        for cond in conditions.iter() {
            array.push(&JsValue::from(cond.clone()));
        }
        array
    }

    #[wasm_bindgen(js_name = wait)]
    pub async fn wait(&self) -> Result<js_sys::Array> {
        let attached: Vec<Condition> = self.conditions.borrow().clone();
        if attached.is_empty() {
            return Err(into_js_error(
                dust_dds::infrastructure::error::DdsError::PreconditionNotMet(String::from(
                    "WaitSet has no attached conditions",
                )),
            ));
        }

        let mut waitset_async = dust_dds::dds_async::wait_set::WaitSetAsync::new();
        for c in &attached {
            waitset_async
                .attach_condition(c.inner.clone())
                .await
                .map_err(into_js_error)?;
        }

        let _ = waitset_async.wait().await.map_err(into_js_error)?;

        let triggered = js_sys::Array::new();
        for c in attached {
            if c.get_trigger_value().await? {
                triggered.push(&JsValue::from(c));
            }
        }
        Ok(triggered)
    }
}
