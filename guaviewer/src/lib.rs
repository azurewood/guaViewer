pub mod app;
pub use app::app_run;

/// cargo check --target aarch64-linux-android
/// cargo apk run --target aarch64-linux-android --lib
/// adb push ../target/debug/apk/guaviewer.apk /sdcard/
#[cfg(target_os = "android")]
mod android_entry {
    use super::*; // Allows access to app_run

    #[unsafe(no_mangle)]
    fn android_main(app: slint::android::AndroidApp) {
        slint::android::init(app).expect("slint android init failed");
        app_run::run_app().expect("run_app failed");
    }
}
