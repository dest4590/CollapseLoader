import os
import zipfile

import requests
from tqdm import tqdm

from .Logger import logger


class DataManager:
    """Used to manage loader data"""

    def __init__(self) -> None:
        self.root_dir = 'data/'
        self.server = 'https://cdn.collapseloader.org/'
        self.server_fallback = 'https://example.com'
        self.repo = 'https://github.com/dest4590/CollapseLoader/'
        self.version = '1.2.4'
        self.session = requests.Session()

        if not os.path.isdir(self.root_dir):
            os.makedirs(self.root_dir)
            logger.debug('Created root dir')

        try:
            r = self.session.get(self.server + 'index.html', timeout=3)
            if 'is not available in your location' in r.text: # If server is blocked
                self.server = self.server_fallback
            logger.debug('Using the main server')
        except requests.exceptions.RequestException:
            logger.debug("The main server is down/inaccessible, we're using fallback")
            self.server = self.server_fallback # Uses a fallback server if the main server is down
        logger.debug('Initialized DataManager')

    def get_local(self, path: str) -> str:
        """Get file locally"""
        return os.path.join(self.root_dir, path)
    
    def get_url(self, path: str) -> str:
        """Gets a link from the web, uses a fallback server if the main one is down"""
        return self.server + path
    
    def download(self, path: str, destination: str = None, mod: bool = False) -> None:
        logger.debug(f'Downloading {path}')

        filename = os.path.basename(path)
        jar = os.path.splitext(filename)[0] + '.jar'
        path_dir = os.path.join(self.root_dir, os.path.splitext(filename)[0])

        dest = destination if destination else os.path.join(self.root_dir, filename)
        # fabric_folder = self.get_local('fabric-loader-0.15.9-1.20.4/mods/')
        
        if not filename.endswith('.jar') and not os.path.exists(data.get_local(filename)):
            if os.path.isdir(path_dir):
                logger.debug(f'{path} Already downloaded, skip')
                return
            else:
                os.makedirs(path_dir)

        elif filename.endswith('.jar'):
            if os.path.exists(os.path.join(path_dir, jar)):
                logger.debug(f'{path} file downloaded, skip')
                return
            if not os.path.isdir(path_dir) and not mod:
                os.makedirs(path_dir)
            
        headers = {}
        if os.path.exists(dest):
            resume_header = {'Range': 'bytes=%d-' % os.path.getsize(dest)}
            headers.update(resume_header)

        response = self.session.get(self.server + filename, headers=headers, stream=True)
        total_size = int(response.headers.get('content-length', 0))

        with tqdm(total=total_size, desc=path, unit="B", unit_scale=True, ascii=True, ncols=100, colour='blue') as progressbar:
            with open(dest, "ab") as f:
                for d in response.iter_content(1024):
                    f.write(d)
                    progressbar.update(len(d))

        if filename.endswith('.zip'):
            with zipfile.ZipFile(dest, 'r') as zip_file:
                zip_file.extractall(path_dir)
            os.remove(dest)
        elif filename.endswith('.jar'):
            os.rename(dest, os.path.join(path_dir, filename))

data = DataManager()
