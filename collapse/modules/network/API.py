import requests

from ...static import API_URL, LOCAL_API
from ..network.Network import network
from ..utils.Module import Module


class API(Module):
    """Class for API requests"""

    def __init__(self, server: str = API_URL) -> None:
        super().__init__()
        self.server = server if not LOCAL_API else 'http://127.0.0.1:8000/'

    def get(self, path: str, prefix: bool = True) -> requests.Response:
        """Makes an API request"""
        url = f'{self.server}{"api/" if prefix else ""}{path}'
        self.debug(f'API request to {path}')
        
        try:
            response = network.get(url)
            response.raise_for_status()
            return response
        except requests.exceptions.RequestException as e:
            self.error(f'API request error: {e}')
            return None

api = API()