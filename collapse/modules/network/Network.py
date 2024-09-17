import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from ..utils.Module import Module


class Network(Module):
    """Module for network operations"""

    def __init__(self):
        super().__init__()
        
        self.session = requests.Session()
        retries = Retry(total=5, backoff_factor=0.1, status_forcelist=[500, 502, 503, 504])
        adapter = HTTPAdapter(max_retries=retries)
        self.session.mount('http://', adapter)
        self.session.mount('https://', adapter)

    def get(self, url, params=None, headers=None, stream=False):
        """Make a GET request to the given URL"""
        try:
            response = self.session.get(url, params=params, headers=headers, stream=stream, timeout=1)
            response.raise_for_status()
            return response
        except requests.exceptions.RequestException as e:
            self.error(f"An error occurred: {e}")
            raise e
 
    def close(self):
        self.session.close()

network = Network()