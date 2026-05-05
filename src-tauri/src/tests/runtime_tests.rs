use crate::core::app_runtime::{DeepLinkAction, DeepLinkDeduplicator, TrayMenuAction};

#[test]
fn tray_menu_action_parses_show() {
    assert!(matches!(TrayMenuAction::parse("show"), TrayMenuAction::Show));
}

#[test]
fn tray_menu_action_parses_launch_client() {
    assert!(matches!(
        TrayMenuAction::parse("launch_42"),
        TrayMenuAction::LaunchClient(42)
    ));
}

#[test]
fn tray_menu_action_ignores_invalid_launch_id() {
    assert!(matches!(
        TrayMenuAction::parse("launch_invalid"),
        TrayMenuAction::Ignore
    ));
}

#[test]
fn deep_link_action_parses_verify_email() {
    let action = DeepLinkAction::parse("collapseloader://verify?code=abc123&email=user@example.com");

    assert!(matches!(
        action,
        Some(DeepLinkAction::VerifyEmail { code, email })
        if code == "abc123" && email == "user@example.com"
    ));
}

#[test]
fn deep_link_action_parses_launch_client() {
    let action = DeepLinkAction::parse("collapseloader://launch?client=7");

    assert!(matches!(
        action,
        Some(DeepLinkAction::LaunchClient { client_id }) if client_id == "7"
    ));
}

#[test]
fn deep_link_action_requires_verify_code() {
    assert!(DeepLinkAction::parse("collapseloader://verify?email=user@example.com").is_none());
}

#[test]
fn deep_link_deduplicator_blocks_immediate_duplicate_url() {
    let url = "collapseloader://verify?code=duplicate-runtime-test";

    assert!(DeepLinkDeduplicator::should_handle(url));
    assert!(!DeepLinkDeduplicator::should_handle(url));
}

#[test]
fn deep_link_deduplicator_allows_different_urls() {
    let first = "collapseloader://verify?code=runtime-first";
    let second = "collapseloader://verify?code=runtime-second";

    assert!(DeepLinkDeduplicator::should_handle(first));
    assert!(DeepLinkDeduplicator::should_handle(second));
}