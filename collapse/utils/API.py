import time

import requests

from .Logger import API as API_LVL
from .Logger import logger


class API:
    def __init__(self, server: str = 'https://web.collapseloader.org/', local: bool = False):
        self.server = server

        if local:
            self.server = 'http://127.0.0.1:8000/'

        self.check_api()

    def get(self, path: str) -> requests.Response:
        logger.log(API_LVL, f'API request to {path}')
        return requests.get(self.server + f'api/{path}')
    
    def clients(self) -> list:
        return self.get('clients/').json()
    
    def check_api(self) -> bool:
        try:
            start_time = time.time()
            r = requests.get(self.server + 'api/clients', timeout=3)
            end_time = time.time()
            ping = round((end_time - start_time) * 1000, 2)

            logger.log(API_LVL, f'Server code: {r.status_code}, Ping: {ping} ms')
        except requests.exceptions.RequestException as e:
            logger.error('API is down, or you are having connectivity problems, check your internet connection and restart loader.')
            input('Press enter >> ')

api = API('https://web.collapseloader.org/', local=False)