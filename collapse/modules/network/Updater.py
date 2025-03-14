import webbrowser

import requests

from ...arguments import args
from ...config import REPOSITORY
from ..network.Network import NameResolutionError, network
from ..render.CLI import selector
from ..storage.Data import data
from ..utils.Language import lang
from ..utils.Module import Module


class Updater(Module):
    """Handles checking for updates and opening download pages"""

    def __init__(self) -> None:
        super().__init__()
        self.latest_releases = []
        self.latest_release = None
        self.remote_version = None
        self.latest_commit = None
        self.local_version = data.version

        if args.disable_updater:
            return

        self.initialize()

    def initialize(self):
        try:
            self.latest_releases = self.get_latest_releases()
            if self.latest_releases:
                self.latest_release = self.latest_releases[0]
                self.remote_version = self.get_remote_version()
            self.latest_commit = self.get_latest_commit()

            if self.remote_version:
                self.debug(
                    lang.t("updater.version-check").format(
                        self.remote_version, self.local_version
                    )
                )
            if self.latest_commit:
                self.debug(lang.t("updater.latest-commit").format(self.latest_commit))

        except requests.exceptions.RequestException as e:
            if "NameResolutionError" in str(e):
                raise NameResolutionError
            self.remote_version = None
            self.latest_commit = None

    def api_request(self, path: str, params: dict = None) -> dict:
        """Makes a request to the GitHub API"""
        url = f"https://api.github.com/repos/{REPOSITORY}/{path}"
        try:
            response = network.get(url, params=params)
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            if "rate limit exceeded" in str(e).lower():
                self.warn(lang.t("updater.rate-limit"))
            else:
                self.error(lang.t("updater.fetch-error").format(path, e))
            raise

    def get_latest_releases(self) -> list:
        """Fetch releases from the GitHub API"""
        return self.api_request("releases")

    def get_remote_version(self) -> str:
        """Fetch the latest remote version from the GitHub API without considering pre-releases"""
        latest_release = next(
            (
                release
                for release in self.latest_releases
                if not release.get("prerelease")
            ),
            None,
        )
        return latest_release.get("tag_name") if latest_release else None

    def get_latest_commit(self) -> str:
        """Fetch the latest commit SHA from the GitHub API"""
        commits = self.api_request("commits", {"per_page": 1})
        if commits:
            return commits[0].get("sha", "")[:7]
        return None

    def check_version(self) -> None:
        """Check if the local version is up to date with the remote version"""
        if self.remote_version and self.remote_version > self.local_version:
            self.info(lang.t("updater.update-notify"))

            if selector.ask(lang.t("updater.update-ask")):
                if self.latest_releases:
                    download_url = None

                    if selector.ask(lang.t("updater.update-ask-dev")):
                        self.debug(lang.t("updater.opening-latest-prerelease"))
                        latest_prerelease = next(
                            (
                                release
                                for release in self.latest_releases
                                if release.get("prerelease")
                            ),
                            None,
                        )
                        if latest_prerelease and latest_prerelease.get("assets"):
                            download_url = latest_prerelease["assets"][0].get(
                                "browser_download_url"
                            )
                    else:
                        self.debug(lang.t("updater.opening-latest-release"))

                        if self.latest_release and self.latest_release.get("assets"):
                            download_url = self.latest_release["assets"][0].get(
                                "browser_download_url"
                            )

                    if download_url:
                        webbrowser.open(download_url)
                else:
                    self.warn(lang.t("updater.no-releases"))


updater = Updater()
