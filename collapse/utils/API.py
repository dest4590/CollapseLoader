import requests

from ..static import LOCAL_API, API_URL
from .Logger import logger


class API:
    """Сlass for API requests"""

    def __init__(self, server: str = API_URL):
        self.server = 'http://127.0.0.1:8000/' if LOCAL_API else server
        self.session = requests.Session()

    def get(self, path: str) -> requests.Response:
        """Makes an API request"""
        url = f'{self.server}api/{path}'
        logger.api(f'API request to {path}')

        try:
            return self.session.get(url)
        except (requests.exceptions.RequestException, requests.exceptions.ConnectionError) as e:
            logger.error(f'API request error: {e}')
            return None


api = API()
