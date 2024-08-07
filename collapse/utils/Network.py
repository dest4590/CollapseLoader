import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

class Network:
    def __init__(self):
        self.session = requests.Session()
        retries = Retry(total=5, backoff_factor=0.1, status_forcelist=[500, 502, 503, 504])
        adapter = HTTPAdapter(max_retries=retries)
        self.session.mount('http://', adapter)
        self.session.mount('https://', adapter)

    def get(self, url, params=None, headers=None, stream=False):
        try:
            response = self.session.get(url, params=params, headers=headers, stream=stream, timeout=5)
            response.raise_for_status()
            return response
        except requests.exceptions.RequestException as e:
            print(f"An error occurred: {e}")
            return None
 
    def close(self):
        self.session.close()

network = Network()