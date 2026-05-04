use crate::core::storage::settings::{Setting, Settings};

#[test]
fn default_settings_have_expected_values() {
    let s = Settings::default();

    assert_eq!(s.ram.value, 2048u32);
    assert_eq!(s.ram.show, true);

    assert_eq!(s.theme.value, "dark".to_string());
    assert_eq!(s.theme.show, false);

    assert_eq!(s.language.value, "en".to_string());
    assert_eq!(s.language.show, true);
}

#[test]
fn from_input_applies_visibility_defaults_and_sets_path() {
    let mut input = Settings::default();

    // flip visibility values to ensure `from_input` resets them
    input.ram.show = false;
    input.theme.show = true;

    let path = Settings::config_path();
    let s = Settings::from_input(input, path.clone());

    // visibility defaults should be applied (ram true, theme false)
    assert_eq!(s.ram.show, true);
    assert_eq!(s.theme.show, false);

    // config_path should be set to provided path
    assert_eq!(s.config_path, path);
}

#[test]
fn setting_deref_and_display() {
    let st: Setting<u32> = Setting::new(123, true);
    // deref
    assert_eq!(*st, 123u32);
    // display
    assert_eq!(format!("{}", st), "123");
}
