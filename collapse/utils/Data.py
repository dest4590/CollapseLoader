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
        self.server_fallback = 'https://сdn-ru.collapseloader.org/'
        self.repo = 'https://github.com/dest4590/CollapseLoader/'
        self.version = '1.2.5'
        self.session = requests.Session()

        if not os.path.isdir(self.root_dir):
            os.makedirs(self.root_dir)
            logger.debug('Created root dir')

        try:
            r = self.session.get(self.server + 'index.html', timeout=3)
            if 'is not available in your location' in r.text:  # If server is blocked
                self.server = self.server_fallback
            logger.debug('Using the main server')
        except requests.exceptions.RequestException:
            logger.debug("The main server is down/inaccessible, using fallback")
            self.server = self.server_fallback  # Uses a fallback server if the main server is down

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

        if not filename.endswith('.jar') and not os.path.exists(self.get_local(filename)):
            if os.path.isdir(path_dir):
                logger.debug(f'{path} already downloaded, skip')
                return
            os.makedirs(path_dir, exist_ok=True)
        elif filename.endswith('.jar'):
            if os.path.exists(os.path.join(path_dir, jar)):
                logger.debug(f'{path} file already downloaded, skip')
                return
            if not os.path.isdir(path_dir) and not mod:
                os.makedirs(path_dir, exist_ok=True)

        headers = {}
        
        if os.path.exists(dest):
            headers['Range'] = f'bytes={os.path.getsize(dest)}-'

        try:
            response = self.session.get(self.server + filename, headers=headers, stream=True)
            response.raise_for_status()
            total_size = int(response.headers.get('content-length', 0))
        except requests.exceptions.RequestException as e:
            logger.error(f"Failed to download {path}: {e}")
            return

        with tqdm(total=total_size, desc=path, unit="B", unit_scale=True, ascii=True, ncols=100, colour='blue') as progressbar:
            with open(dest, "ab") as f:
                for chunk in response.iter_content(1024):
                    if chunk:
                        f.write(chunk)
                        progressbar.update(len(chunk))

        try:
            if filename.endswith('.zip'):
                with zipfile.ZipFile(dest, 'r') as zip_file:
                    zip_file.extractall(path_dir)
                os.remove(dest)
            elif filename.endswith('.jar'):
                os.rename(dest, os.path.join(path_dir, filename))
        except (zipfile.BadZipFile, OSError) as e:
            logger.error(f"Error processing {dest}: {e}")
            if os.path.exists(dest):
                os.remove(dest)

data = DataManager()