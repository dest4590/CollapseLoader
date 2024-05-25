import requests
from urllib3.exceptions import MaxRetryError, NameResolutionError
from requests.exceptions import ConnectionError

from .Logger import API as API_LVL
from .Logger import logger

class API:
    def __init__(self, server: str = 'https://web.collapseloader.org/', local: bool = False):
        self.server = 'http://127.0.0.1:8000/' if local else server
        self.session = requests.Session()

    def get(self, path: str) -> requests.Response:
        url = f'{self.server}api/{path}'
        logger.log(API_LVL, f'API request to {path}')
        try:
            return self.session.get(url)
        except (ConnectionError, MaxRetryError, NameResolutionError) as e:
            logger.error(f'Failed to reach {url}: {e}')
            raise
        except requests.exceptions.RequestException as e:
            logger.error(f'API request error: {e}')
            raise
    
    def clients(self) -> list:
        try:
            response = self.get('clients/')
            return response.json()
        except (ConnectionError, MaxRetryError, NameResolutionError) as e:
            logger.error(f'Error fetching clients: {e}')
            return []
        except requests.exceptions.RequestException as e:
            logger.error(f'API request error: {e}')
            return []

api = API('https://web.collapseloader.org/', local=False)
