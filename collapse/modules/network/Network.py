import requests

from ...arguments import args
from ..utils.Language import lang
from ..utils.Module import Module


class Network(Module):
    """Module for network operations"""

    def __init__(self):
        super().__init__()
        
        self.session = requests.Session()
        self.timeout = args.timeout if args.timeout else 5

    def get(self, url, params=None, headers=None, stream=False):
        """Make a GET request to the given URL"""
        try:
            response = self.session.get(url, params=params, headers=headers, stream=stream, timeout=self.timeout)
            return response
        except requests.exceptions.RequestException as e:
            self.error(lang.t('network.error').format(e))
            raise e
 
    def close(self):
        self.session.close()

network = Network()