use core::future::Future;
use dust_dds::{
    infrastructure::time::Time,
    runtime::{Clock, DdsRuntime, Spawner, TaskHandle, Timer},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(handler: &js_sys::Function, timeout: i32) -> i32;
}

#[derive(Clone, Default)]
pub struct WasmClock;

impl Clock for WasmClock {
    fn now(&self) -> Time {
        let millis = js_sys::Date::now();
        let sec = (millis / 1000.0) as i32;
        let nanosec = ((millis % 1000.0) * 1_000_000.0) as u32;
        Time::new(sec, nanosec)
    }
}

#[derive(Clone, Default)]
pub struct WasmTimer;

impl Timer for WasmTimer {
    fn delay(&mut self, duration: core::time::Duration) -> impl Future<Output = ()> + Send {
        let (sender, receiver) = futures_channel::oneshot::channel::<()>();
        let cb = Closure::once_into_js(move || {
            let _ = sender.send(());
        });
        let ms = duration.as_millis().min(i32::MAX as u128) as i32;
        set_timeout(cb.unchecked_ref(), ms);
        async move {
            let _ = receiver.await;
        }
    }
}

pub async fn sleep_ms(ms: i32) {
    let (sender, receiver) = futures_channel::oneshot::channel::<()>();
    let cb = Closure::once_into_js(move || {
        let _ = sender.send(());
    });
    set_timeout(cb.unchecked_ref(), ms);
    let _ = receiver.await;
}

#[derive(Clone, Default)]
pub struct WasmSpawner;

pub struct WasmTaskHandle;

impl TaskHandle for WasmTaskHandle {
    fn join(&self) {}
}

impl Spawner for WasmSpawner {
    type TaskHandle = WasmTaskHandle;

    fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) -> Self::TaskHandle {
        wasm_bindgen_futures::spawn_local(f);
        WasmTaskHandle
    }
}

#[derive(Clone, Default)]
pub struct WasmRuntime;

impl DdsRuntime for WasmRuntime {
    type ClockHandle = WasmClock;
    type TimerHandle = WasmTimer;
    type SpawnerHandle = WasmSpawner;

    fn timer(&self) -> Self::TimerHandle {
        WasmTimer
    }

    fn clock(&self) -> Self::ClockHandle {
        WasmClock
    }

    fn spawner(&self) -> Self::SpawnerHandle {
        WasmSpawner
    }
}
