use crate::commands::updater::{
    compare_versions, extract_changelog_json_block, parse_changelog_and_translations,
    parse_version, parse_version_component, truncate_str,
};
use std::cmp::Ordering;

#[test]
fn parse_version_accepts_prefixed_and_suffixed_versions() {
    assert_eq!(parse_version("v1.2.3-beta.1").unwrap(), (1, 2, 3));
}

#[test]
fn parse_version_rejects_invalid_format() {
    assert_eq!(parse_version("1.2").unwrap_err(), "Invalid version format");
}

#[test]
fn parse_version_component_reports_label() {
    assert_eq!(
        parse_version_component("abc", "minor").unwrap_err(),
        "Invalid minor version"
    );
}

#[test]
fn compare_versions_orders_versions_correctly() {
    assert_eq!(compare_versions("1.2.3", "1.3.0").unwrap(), Ordering::Less);
    assert_eq!(compare_versions("2.0.0", "1.9.9").unwrap(), Ordering::Greater);
    assert_eq!(compare_versions("1.2.3", "1.2.3").unwrap(), Ordering::Equal);
}

#[test]
fn truncate_str_keeps_short_strings() {
    assert_eq!(truncate_str("short", 10), "short");
}

#[test]
fn truncate_str_marks_truncated_strings() {
    assert_eq!(truncate_str("abcdefgh", 5), "abcde...<truncated 3 chars>");
}

#[test]
fn extract_changelog_json_block_returns_embedded_json() {
    let body = "intro\n```changelog\n{\"entries\":[]}\n```\noutro";

    assert_eq!(
        extract_changelog_json_block(body),
        Some("{\"entries\":[]}".to_string())
    );
}

#[test]
fn parse_changelog_and_translations_reads_both_sections() {
    let body = r#"
    {
        "entries": [
            {
                "version": "1.2.3",
                "changes": [
                    {
                        "category": "feature",
                        "description_key": "updates.feature.one",
                        "icon": "sparkles"
                    }
                ],
                "date": "2026-05-05",
                "highlights": ["Faster startup"]
            }
        ],
        "translations": {
            "en": {
                "updates.feature.one": "Feature one"
            }
        }
    }
    "#;

    let (changelog, translations) =
        parse_changelog_and_translations(body).expect("changelog should parse");

    assert_eq!(changelog.len(), 1);
    assert_eq!(changelog[0].version, "1.2.3");
    assert_eq!(changelog[0].highlights, vec!["Faster startup"]);
    assert_eq!(
        translations
            .expect("translations should exist")["en"]["updates.feature.one"],
        "Feature one"
    );
}