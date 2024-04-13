from .Logger import logger
from tqdm import tqdm
import requests
import zipfile
import os

class DataManager:
    """Used to manage loader data"""

    def __init__(self) -> None:
        self.root_dir = 'data/'
        self.server = 'https://storage.googleapis.com/collapseloader/'
        self.server_fallback =  'https://loader.collapseloader.org'
        self.server_assets = 'https://axkanxneklh7.objectstorage.eu-amsterdam-1.oci.customer-oci.com/n/axkanxneklh7/b/assets/o/'
        self.repo = 'https://github.com/dest4590/CollapseLoader/'
        self.version = '1.2.3'

        if not os.path.isdir(self.root_dir):
            os.mkdir(self.root_dir)
            logger.debug('Created root dir')

        try:
            r = requests.get(self.server, timeout=3)

            if 'is not available in your location' in r.text: # If server is blocked
                self.server = self.server_fallback

            logger.debug('Using the main server')
            
        except requests.exceptions.RequestException:
            logger.debug("The main server is down/inaccessible, we're using fallback")
            self.server = self.server_fallback # Uses a fallback server if the main server is down

        logger.debug('Initialized DataManager')

    def get_local(self, path: str) -> str:
        """Get file locally"""
        return self.root_dir + path
    
    def get_url(self, path: str) -> str:
        """Gets a link from the web, uses a fallback server if the main one is down"""
        return self.server + path
    
    def download(self, path: str, destination: str = None, mod: bool = False) -> True:
        logger.debug(f'Downloading {path}')

        filename = os.path.basename(path)
        jar = os.path.splitext(filename)[0] + '.jar'
        path = self.root_dir + filename 
        path_dir = self.root_dir + os.path.splitext(filename)[0] + '/'
        dest = destination if destination != None else self.root_dir + filename
        fabric_folder = data.get_local('fabric-loader-0.15.9-1.20.4') + '/mods/'
        
        if not filename.endswith('.jar'):
            if os.path.isdir(path_dir):
                logger.debug(f'{path} Already downloaded, skip')
                return
            
            else:
                os.mkdir(path_dir)

        elif filename.endswith('.jar'):
            if os.path.exists(path_dir + jar) or os.path.exists(fabric_folder + filename):
                logger.debug(f'{path} file downloaded, skip')
                return
            
            if not os.path.isdir(path_dir) and not mod:
                os.mkdir(path_dir)

            
        response = requests.get(self.server + filename, stream=True)
 
        total_size = int(response.headers.get('content-length', 0))

        with tqdm(total=total_size, desc=path, unit="B", unit_scale=True, ascii=True, ncols=100, colour='blue') as progressbar:
            with open(dest, "wb") as f:
                for d in response.iter_content(1024):
                    f.write(d)
                    progressbar.update(len(d))

        if not mod:
            if filename.endswith('.zip'):
                with zipfile.ZipFile(dest, 'r') as zip_file:
                    zip_file.extractall(path_dir)

                os.remove(dest)

            if filename.endswith('.jar'):
                os.rename(dest, path_dir + filename)
        else:
            logger.debug('Installing mod')

            
            if not os.path.exists(fabric_folder):
                os.mkdir(fabric_folder)

            if not os.path.exists(fabric_folder + filename):
                os.rename(dest, data.get_local('fabric-loader-0.15.9-1.20.4') + '/mods/' + filename)
    
data = DataManager()