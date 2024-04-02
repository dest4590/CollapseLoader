from ..logger import logger
import requests
import os

class DataManager:
    """Used to manage loader data"""

    def __init__(self):
        self.root_dir = 'data/'
        self.server = 'https://loader.dest4590.lol/'  

        if not os.path.isdir(self.root_dir):
            os.mkdir(self.root_dir)
            logger.debug('Created root dir')

        try:
            requests.get(self.server, timeout=3)
            logger.debug('Using the main server')
            
        except requests.exceptions.RequestException:
            logger.debug("The main server is down, we're using fallback")
            self.server = 'https://loader-fallback.dest4590.lol/' # Uses a fallback server if the main server is down

        logger.debug('Initialized DataManager')

    def get_local(self, path: str):
        """Get file locally"""
        return self.root_dir + path
    
    def get_url(self, path: str):
        """Gets a link from the web, uses a fallback server if the main one is down"""
        return self.server + path
    
data = DataManager()