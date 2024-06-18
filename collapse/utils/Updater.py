import webbrowser

import requests

from .Data import data
from .Logger import logger
from .CLI import selector


class Updater:
    def __init__(self):
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

    def get_remote_version(self) -> str:
        try:
            response = requests.get('https://api.github.com/repos/dest4590/CollapseLoader/releases/latest', timeout=5)
            response.raise_for_status()
            return response.json().get('tag_name')
        except requests.exceptions.RequestException as e:
            logger.error(f'Failed to fetch remote version: {e}')
            raise

    def get_latest_commit(self) -> str:
        try:
            response = requests.get('https://api.github.com/repos/dest4590/CollapseLoader/commits', params={'per_page': 1}, timeout=5)
            response.raise_for_status()
            return response.json()[0].get('sha', '')[:7]
        except requests.exceptions.RequestException as e:
            logger.error(f'Failed to fetch latest commit: {e}')
            raise

    def check_version(self):
        if self.remote_version and self.remote_version > self.local_version:
            logger.info('Update your loader!')

            if selector.ask('Open a download page (y,n)'):
                webbrowser.open(data.repo + 'releases/latest')


updater = Updater()