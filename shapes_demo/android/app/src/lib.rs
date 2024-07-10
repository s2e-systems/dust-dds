#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use dust_dds_shapes_demo::eframe;
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(android_logger::Config::default());
    log::info!("shapes demo");

    let options = eframe::NativeOptions {
        event_loop_builder: Some(Box::new(move |builder| {
            builder.with_android_app(app);
        })),
        default_theme: eframe::Theme::Light,
        ..Default::default()
    };

    eframe::run_native(
        "Dust DDS Shapes Demo",
        options,
        Box::new(|_cc| Ok(Box::<dust_dds_shapes_demo::app::ShapesDemoApp>::default())),
    )
    .unwrap_or_else(|err| {
        log::error!("Failure while running EFrame application: {err:?}");
    });
}
