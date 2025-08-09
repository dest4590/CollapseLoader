When changing the version, you need to change three files:

-   package.json
-   src-tauri/Cargo.toml
-   src-tauri/tauri.conf.json

Put changelog into [updater.rs](src-tauri/src/commands/updater.rs) file
Write translations for changelogs in [locales](src/i18n/locales)

For example:

if you need to add translation for this ChangeItem:

```rust
ChangeItem {
    category: Category::Bugfix,
    description: "fixed_settings".to_string(),
    icon: "🎉".to_string(),
}
```

You need to add this to the [locales](src/i18n/locales) files:

```json
{
    "updater": {
        "changelogs": {
            "bugfix": {
                "v0_1_7": {
                    "fixed_settings": "Fixed settings issue"
                }
            }
        }
    }
}
```
