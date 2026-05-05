CollapseLoader Reborn (1.1.0-LB1.1.0)

<< CHANGELOG >>
Minecraft account nickname validation (3–16 chars, a-z 0-9 _)
Client screenshots loaded from CDN 
Fixed Fabric classpath – removed conflicting patchy/netty from vanilla libraries
Fixed custom Fabric client assets dir (wrong asset index)
Hidden cloud sync, local account sync and telemetry in settings

<< LINKS >>

[Telegram](https://t.me/collapseloader)

<details>
<summary><strong>do not read, for internal updater</strong></summary>

```json
{
    "entries": [
        {
            "version": "v1.1.0-LB1.1.0",
            "date": "2026-04-27",
            "highlights": [
                "Minecraft account nickname validation",
                "Client screenshots from CDN",
                "Fabric classpath fix",
                "Custom Fabric assets fix"
            ],
            "changes": [
                {
                    "category": "feature",
                    "description_key": "updater.changelogs.feature.v1_1_0.account_validation",
                    "icon": "check"
                },
                {
                    "category": "feature",
                    "description_key": "updater.changelogs.feature.v1_1_0.screenshots",
                    "icon": "image"
                },
                {
                    "category": "bugfix",
                    "description_key": "updater.changelogs.bugfix.v1_1_0.fabric_classpath",
                    "icon": "wrench"
                },
                {
                    "category": "bugfix",
                    "description_key": "updater.changelogs.bugfix.v1_1_0.fabric_assets",
                    "icon": "folder"
                },
                {
                    "category": "improvement",
                    "description_key": "updater.changelogs.improvement.v1_1_0.settings_cleanup",
                    "icon": "clean"
                }
            ]
        }
    ],
    "translations": {
        "en": {
            "updater": {
                "categories": {
                    "feature": "Feature",
                    "improvement": "Improvement",
                    "bugfix": "Bugfix",
                    "refactor": "Refactor",
                    "other": "Other"
                },
                "changelogs": {
                    "feature": {
                        "v1_1_0": {
                            "account_validation": "Minecraft account nickname validation – 3–16 chars, only a-z 0-9 _",
                            "screenshots": "Client screenshots loaded from CDN"
                        }
                    },
                    "bugfix": {
                        "v1_1_0": {
                            "fabric_classpath": "Fixed Fabric classpath – removed conflicting patchy and netty from vanilla libraries",
                            "fabric_assets": "Fixed custom Fabric client using wrong asset index (17 instead of 1.21)"
                        }
                    },
                    "improvement": {
                        "v1_1_0": {
                            "settings_cleanup": "Hidden cloud sync, local account sync and telemetry options in settings"
                        }
                    }
                }
            }
        }
    }
}
```

</details>
