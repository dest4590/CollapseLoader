use native_dialog::{DialogBuilder, MessageLevel};

pub fn show_confirm(title: &str, text: &str) -> bool {
    DialogBuilder::message()
        .set_level(MessageLevel::Info)
        .set_title(title)
        .set_text(text)
        .confirm()
        .show()
        .unwrap_or(false)
}

pub fn show_message(title: &str, text: &str) {
    let _ = DialogBuilder::message()
        .set_level(MessageLevel::Info)
        .set_title(title)
        .set_text(text)
        .alert()
        .show();
}

pub fn show_error(title: &str, text: &str) {
    let _ = DialogBuilder::message()
        .set_level(MessageLevel::Error)
        .set_title(title)
        .set_text(text)
        .alert()
        .show();
}
