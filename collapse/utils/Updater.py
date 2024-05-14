import webbrowser

import requests

from .Data import data
from .Logger import logger
from .Selector import selector


class Updater:
    def __init__(self):
        self.remote_version = requests.get('https://api.github.com/repos/dest4590/CollapseLoader/releases/latest').json()['tag_name']
        self.latest_commit = requests.get('https://api.github.com/repos/dest4590/CollapseLoader/commits', data={'per_page': 1}).json()[0]['sha'][:7]
        self.local_version = data.version

        logger.debug(f'Remote: {self.remote_version}, local: {self.local_version}')
        logger.debug(f'Latest commit: {self.latest_commit}')

    def check_version(self):
        if self.remote_version > self.local_version:
            logger.info('Update your loader!')

            if selector.ask('Open a download page [y,n]'):
                webbrowser.open(data.repo + 'releases/latest')
            
            return

updater = Updater()