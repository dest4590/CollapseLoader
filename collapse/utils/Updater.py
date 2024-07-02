import webbrowser

import requests

from .CLI import selector
from .Data import data
from .Logger import logger


class Updater:
    """Handles checking for updates and opening download pages"""

    def __init__(self) -> None:
        try:
            self.latest_releases = self.get_latest_releases()
            self.latest_release = self.latest_releases[0]
            self.remote_version = self.get_remote_version()
            self.latest_commit = self.get_latest_commit()
            self.local_version = data.version

            logger.debug(f'Remote: {self.remote_version}, local: {self.local_version}')
            logger.debug(f'Latest commit: {self.latest_commit}')
        except requests.exceptions.RequestException as e:
            logger.error(f'Error initializing Updater: {e}')
            self.remote_version = None
            self.latest_commit = None
            self.local_version = data.version

    def api_request(self, path: str, params: dict = None) -> dict:
        """Makes a request to the GitHub API"""
        try:
            response = requests.get('https://api.github.com/repos/dest4590/CollapseLoader/' + path,
                                     timeout=5, params=params)

            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            logger.error(f'Failed to fetch {path}: {e}')
            raise

    def get_latest_releases(self) -> dict:
        """Fetch releases from the GitHub API"""
        return self.api_request('releases')

    def get_remote_version(self) -> str:
        """Fetch the latest remote version from the GitHub API without considering pre-releases"""
        latest_release = next((release for release in self.latest_releases if not release.get('prerelease')), None)
        if latest_release:
            return latest_release.get('tag_name')
        return None

    def get_latest_commit(self) -> str:
        """Fetch the latest commit SHA from the GitHub API"""
        return self.api_request('commits', {'per_page': 1})[0].get('sha', '')[:7]

    def check_version(self) -> None:
        """Check if the local version is up to date with the remote version"""
        if self.remote_version and self.remote_version > self.local_version:
            logger.info('Update your loader!')

            if selector.ask('Download a new version (y,n)'):
                if self.latest_releases:
                    if selector.ask('Dev version (y,n)'): # Prelease
                        logger.debug('Downloading dev version')
                        latest_prerelease = next((release for release in self.latest_releases if release.get('prerelease')), None)
                        if latest_prerelease and latest_prerelease.get('assets'):
                            webbrowser.open(latest_prerelease['assets'][0].get('browser_download_url'))
                    else: # Release
                        logger.debug('Downloading stable release')
                        latest_release = next((release for release in self.latest_releases if not release.get('prerelease')), None)
                        if latest_release and latest_release.get('assets'):
                            webbrowser.open(latest_release['assets'][0].get('browser_download_url'))
                else:
                    logger.warn('No releases found')

updater = Updater()
