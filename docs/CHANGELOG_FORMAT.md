# Changelog Format for GitHub Releases

This project requires embedding a structured changelog directly in the GitHub Release body. The updater will parse a fenced code block marked with `changelog` and read JSON data describing the changelog entries. There is no longer any bundled or hardcoded changelog in the application; if the release body doesn't contain a valid `changelog` block the changelog list will be empty.

Place a fenced code block in your release notes like this:

```markdown
```changelog
[
  {
    "version": "v0.1.8",
    "date": "2025-08-29",
    "highlights": ["Another public release"],
    "changes": [
      {
        "category": "feature",
        "description_key": "updater.changelogs.feature.v0_1_8.added_x",
        "icon": "✨"
      },
      {
        "category": "bugfix",
        "description_key": "Fixed crash when opening settings",
        "icon": "🐛"
      }
    ]
  }
]
```
```

Rules and recommendations:

- The fenced block must start with exactly ```` ```changelog ```` or ```` ``` changelog ````.
- The content must be valid JSON. It can be either a single object (one changelog entry) or an array of entries.
- Fields in each entry:
  - `version` (string) — release tag or version (e.g. `v0.1.8`).
  - `date` (string, optional) — human-readable date.
  - `highlights` (array of strings, optional) — short highlighted points.
  - `changes` (array) — required, list of change items.
- Fields for each change item:
  - `category` (string) — one of `feature`, `improvement`, `bugfix`, `other` (case-insensitive).
  - `description_key` (string) — either a translation key (recommended) or plain text.
  - `icon` (string, optional) — an emoji or short icon string.

Translation keys inside the release body
--------------------------------------

You can include i18n translations directly in the GitHub release body so contributors can provide localized strings alongside the structured changelog. The updater will resolve translation keys using the app's i18n files; however it's convenient to place translations inside the release body for quick additions or when a translation key should map to different text per release.

To include localized strings inside the same changelog block, use an additional top-level `translations` object. Example:

```changelog
{
  "entries": [
    {
      "version": "v0.1.8",
      "date": "2025-08-29",
      "changes": [
        { "category": "feature", "description_key": "updater.changelogs.feature.v0_1_8.added_auto_parse", "icon": "✨" }
      ]
    }
  ],
  "translations": {
    "en": {
      "updater": {
        "changelogs": {
          "feature": {
            "v0_1_8": { "added_auto_parse": "Added GitHub release changelog parsing" }
          }
        }
      }
    },
    "ru": {
      "updater": {
        "changelogs": {
          "feature": {
            "v0_1_8": { "added_auto_parse": "Добавлен разбор changelog из релиза GitHub" }
          }
        }
      }
    }
  }
}
```

How this works:
- The updater will parse the `entries` array as changelog entries (or accept the top-level array as before).
- If a `translations` object is present, you can extract those translations and merge them into your app's i18n runtime (this requires the frontend to support merging — see next section).

Important: In the current implementation the backend only parses the `entries` (or top-level array) to produce `ChangelogEntry` objects. If you want the app to automatically import the `translations` block into runtime i18n, I can implement a merge: the frontend will receive the translations object and apply them to the i18n instance at runtime. Without that, include full keys and ensure those keys are present in the shipped locale files.

Translation support:

- To allow localized changelog descriptions, prefer using translation keys for `description_key` (dot-separated keys, e.g. `updater.changelogs.feature.v0_1_8.added_x`).
- Add corresponding translations to your locale files (for example `src/i18n/locales/en.json` and `ru.json`) under the `updater.changelogs` tree.
- If `description_key` contains a dot `.`, the frontend will try to resolve it via the i18n system. If no translation is found, the raw key will be shown.

Examples:

1) Using translation keys (recommended):

```changelog
[
  {
    "version": "v0.1.8",
    "date": "2025-08-29",
    "changes": [
      {
        "category": "feature",
        "description_key": "updater.changelogs.feature.v0_1_8.added_x",
        "icon": "✨"
      }
    ]
  }
]
```

Then in `src/i18n/locales/en.json`:

```json
"updater": {
  "changelogs": {
    "feature": {
      "v0_1_8": {
        "added_x": "Added X feature"
      }
    }
  }
}
```

2) Using plain text descriptions (quick and easy):

```changelog
{
  "version": "v0.1.8",
  "changes": [
    { "category": "bugfix", "description_key": "Fixed crash when opening settings", "icon": "🐛" }
  ]
}
```

Notes for maintainers:

- If the release body does not contain a `changelog` fenced block, the updater will fall back to built-in static changelog entries included in the application binary.
 - If the release body does not contain a `changelog` fenced block, the application will not show any changelog entries (there is no built-in fallback anymore). Always include a `changelog` block in releases to ensure users see the changelog.
- Keep translation keys stable across versions when possible to reduce duplication in locale files. You can scope them by version (e.g. `v0_1_8`) or by feature name.