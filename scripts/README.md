This folder contains scripts that are used to automate various tasks related to the project.

## List of Scripts

- [bump_version.py](bump_version.py): A script to automate the process of bumping the version number in the project. It updates version and codename in the relevant files.

- [remove_unused_actions.py](remove_unused_actions.py): A script to delete GitHub Action runs that were unused. It supports a dry-run mode by default and can apply changes with a flag.

- [remove_releases.py](remove_releases.py): A script to delete GitHub releases except the latest one. It supports a dry-run mode by default and can apply changes with a flag.

- [serve_mock_release.py](serve_mock_release.py): A script to serve a mock release file for testing purposes. It uses Python's built-in HTTP server to serve [mock_release.json](../docs/versioning/mock_release.json) with [body.md](../docs/versioning/body.md) contents.

- [md5.py](md5.py): Compute MD5 hash of a file (default: .jar). Interactive file picker or direct mode.

- [new_client.py](new_client.py): Add a new client entry to the CDN JSON. Interactive menu or direct mode.

- [scripts_gui.py](scripts_gui.py): Web GUI with buttons for MD5 and new client. Opens browser at localhost:8765. No dependencies required.
