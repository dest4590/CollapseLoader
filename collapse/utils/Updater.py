import webbrowser

import requests

from .CLI import selector
from .Data import data
from .Logger import logger


class Updater:
    """Handles checking for updates and opening download pages"""

    def __init__(self) -> None:
        try:
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

    def get_remote_version(self) -> str:
        """Fetch the latest remote version from the GitHub API"""
        return self.api_request('releases/latest').get('tag_name')

    def get_latest_commit(self) -> str:
        """Fetch the latest commit SHA from the GitHub API"""
        return self.api_request('commits', {'per_page': 1})[0].get('sha', '')[:7]

    def check_version(self) -> None:
        """Check if the local version is up to date with the remote version"""
        if self.remote_version and self.remote_version > self.local_version:
            logger.info('Update your loader!')

            if selector.ask('Open a download page (y,n)'):
                webbrowser.open(data.repo + 'releases/latest')


updater = Updater()
