import requests

from ...arguments import args
from ...constants import API_URL
from ..network.Network import network
from ..utils.Language import lang
from ..utils.Module import Module


class API(Module):
    """Class for API requests"""

    def __init__(self) -> None:
        super().__init__()
        self.server = args.api_url if args.api_url else API_URL

    def get(self, path: str, prefix: bool = True) -> requests.Response:
        """Makes an API request"""
        if not path.endswith('/') and prefix:
            path += '/'
        
        url = f'{self.server}{"api/" if prefix else ""}{path}'
        self.debug(lang.t('api.request').format(path))
        
        try:
            response = network.get(url)
            response.raise_for_status()
            return response
        except requests.exceptions.RequestException as e:
            self.error(lang.t('api.error').format(e))
            return None

api = API()