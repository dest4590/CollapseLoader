import requests

from ..modules.Module import Module
from ..static import LOCAL_API, API_URL

class API(Module):
    """Сlass for API requests"""

    def __init__(self, server: str = API_URL) -> None:
        super().__init__()
        self.server = 'http://127.0.0.1:8000/' if LOCAL_API else server
        self.session = requests.Session()

    def get(self, path: str) -> requests.Response:
        """Makes an API request"""
        url = f'{self.server}api/{path}'
        self.debug(f'API request to {path}')

        try:
            return self.session.get(url)
        except (requests.exceptions.RequestException, requests.exceptions.ConnectionError) as e:
            self.error(f'API request error: {e}')
            return None


api = API()
