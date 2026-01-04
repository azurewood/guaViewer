pub mod app;
pub use app::app_run;
// #[unsafe(no_mangle)]
// fn android_main(app: slint::android::AndroidApp) {
//     slint::android::init(app).expect("slint android init failed");
//     app_run::run_app().expect("run_app failed");
// }
