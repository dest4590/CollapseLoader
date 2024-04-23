from .Logger import logger
import requests

class API:
    def __init__(self, endpoint: str = 'https://api.collapseloader.org/'):
        self.endpoint = endpoint

    def get(self, path: str) -> requests.Response:
        logger.debug(f'API request to {path}')
        return requests.get(self.endpoint + f'{path}')

    def clients(self) -> list:
        return self.get('clients/?format=json').json()
    
api = API('https://api.collapseloader.org/')