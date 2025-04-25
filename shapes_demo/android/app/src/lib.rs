#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use dust_dds_shapes_demo::eframe;
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(android_logger::Config::default());
    log::info!("shapes demo");

    let ctx = unsafe { jni::objects::JObject::from_raw(app.activity_as_ptr().cast()) };
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) }.unwrap();
    let env = vm.attach_current_thread().unwrap();

    let class_context = env.find_class("android/content/Context").unwrap();
    let wifi_service = env
        .get_static_field(class_context, "WIFI_SERVICE", "Ljava/lang/String;")
        .unwrap();

    let wifi_manager = env
        .call_method(
            ctx,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[wifi_service],
        )
        .unwrap()
        .l()
        .unwrap();

    // wifiManager.createMulticastLock("myLock")
    let lock_tag = env.new_string("rustMulticastLock").unwrap();
    let multicast_lock = env
        .call_method(
            wifi_manager,
            "createMulticastLock",
            "(Ljava/lang/String;)Landroid/net/wifi/WifiManager$MulticastLock;",
            &[lock_tag.into()],
        )
        .unwrap()
        .l()
        .unwrap();

    // multicastLock.acquire()
    env.call_method(multicast_lock, "acquire", "()V", &[])
        .unwrap();

    println!("Multicast lock acquired!");

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
