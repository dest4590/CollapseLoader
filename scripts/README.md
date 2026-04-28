This folder contains scripts that are used to automate various tasks related to the project.

## List of Scripts

- [bump_version.py](bump_version.py): A script to automate the process of bumping the version number in the project. It updates version and codename in the relevant files.

- [remove_canceled_actions.py](remove_canceled_actions.py): A script to delete GitHub Action runs that were canceled. It supports a dry-run mode by default and can apply changes with a flag.

- [remove_releases.py](remove_releases.py): A script to delete GitHub releases except the latest one. It supports a dry-run mode by default and can apply changes with a flag.

- [serve_mock_release.py](serve_mock_release.py): A script to serve a mock release file for testing purposes. It uses Python's built-in HTTP server to serve [mock_release.json](../docs/versioning/mock_release.json) with [body.md](../docs/versioning/body.md) contents.
