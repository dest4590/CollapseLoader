import requests

from ..utils.Module import Module
from ...static import API_URL, LOCAL_API
from ..network.Network import network


class API(Module):
    """Class for API requests"""

    def __init__(self, server: str = API_URL) -> None:
        super().__init__()
        self.server = 'http://127.0.0.1:8000/' if LOCAL_API else server

    def get(self, path: str) -> requests.Response:
        """Makes an API request"""
        url = f'{self.server}api/{path}'
        self.debug(f'API request to {path}')
        
        try:
            response = network.get(url)
            response.raise_for_status()
            return response
        except requests.exceptions.RequestException as e:
            self.error(f'API request error: {e}')
            return None

api = API()