from ..logger import logger
from .Data import data
from tqdm import tqdm
import requests
import zipfile
import os

cheat_list = []

class Cheat:
    def __init__(self, name: str, link: str) -> None:
        self.name = name
        self.link = link
        self.filename = os.path.basename(self.link)
        self.path = data.root_dir + self.filename
        self.path_dir = data.root_dir + os.path.splitext(self.filename)[0] + '/'

        cheat_list.append(self)

    def download(self):
        if not os.path.isdir(self.path):
            logger.warn(f'Client {self.name} already downloaded')
            return

        logger.debug('Downloading client')

        response = requests.get(self.link, stream=True)

        total_size = int(response.headers.get('content-length', 0))

        # Get code from google gemini
        with tqdm(total=total_size, unit="B", unit_scale=True, ascii=True, ncols=100, colour='blue') as progressbar:
            with open(data.root_dir + self.filename, "wb") as f:
                for d in response.iter_content(1024):
                    f.write(d)
                    progressbar.update(len(d))

        logger.info(f'Downloaded client into: {data.root_dir}{self.filename}')

        with zipfile.ZipFile(self.path, 'r') as zip_file:
            zip_file.extractall(self.path_dir)

        logger.debug('Removing zip')
        os.remove(data.root_dir + self.filename)