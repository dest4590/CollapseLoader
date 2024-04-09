import webbrowser
import requests

from .Logger import logger
from .Data import data
from .Selector import selector

class Updater:
    def __init__(self):
        self.remote_version = requests.get('https://api.github.com/repos/dest4590/CollapseLoader/releases/latest').json()['tag_name']
        self.local_version = data.version

        logger.debug(f'Remote: {self.remote_version}, local: {self.local_version}')

    def check_version(self):
        if self.remote_version > self.local_version:
            logger.info('Update your loader!')

            if selector.ask('Open a download page [y,n]'):
                webbrowser.open(data.repo + 'releases/latest')
            
            return

updater = Updater()