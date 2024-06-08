import requests
from requests.exceptions import ConnectionError
from urllib3.exceptions import MaxRetryError, NameResolutionError

from ..static import LOCAL_API
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
        except (ConnectionError, MaxRetryError, NameResolutionError, ConnectionRefusedError) as e:
            logger.error(f'Failed to reach {url}: {e}')
            raise
        except requests.exceptions.RequestException as e:
            logger.error(f'API request error: {e}')
            raise

api = API('https://web.collapseloader.org/', local=LOCAL_API)
