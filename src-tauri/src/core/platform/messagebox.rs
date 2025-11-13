#[cfg(target_os = "windows")]
use native_dialog::{DialogBuilder, MessageLevel};

#[cfg(target_os = "windows")]
pub fn show_confirm(title: &str, text: &str) -> bool {
    DialogBuilder::message()
        .set_level(MessageLevel::Info)
        .set_title(title)
        .set_text(text)
        .confirm()
        .show()
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
pub fn show_message(title: &str, text: &str) {
    let _ = DialogBuilder::message()
        .set_title(title)
        .set_text(text)
        .alert()
        .show();
}

#[cfg(target_os = "windows")]
pub fn show_error(title: &str, text: &str) {
    let _ = DialogBuilder::message()
        .set_level(MessageLevel::Error)
        .set_title(title)
        .set_text(text)
        .alert()
        .show();
}

#[cfg(not(target_os = "windows"))]
pub fn show_error(_title: &str, _text: &str) {}

#[cfg(not(target_os = "windows"))]
pub fn show_confirm(_title: &str, _text: &str) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn show_message(_title: &str, _text: &str) {}
