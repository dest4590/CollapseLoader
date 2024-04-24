from .Logger import logger
import requests

class API:
    def __init__(self, server: str = 'https://api.collapseloader.org/', test: bool = False):
        self.server = server

        if test:
            self.server = 'https://test.collapseloader.org' # this subdomain not exist, you don't have to try it.

        self.check_api()

    def get(self, path: str) -> requests.Response:
        logger.debug(f'API request to {path}')
        return requests.get(self.server + f'api/{path}')
    
    def clients(self) -> list:
        return self.get('clients/').json()
    
    def check_api(self) -> bool:
        try:
            r = requests.get(self.server + '/api/clients', timeout=3)
        except requests.exceptions.RequestException as e:
            logger.error('API is down, or you are having connectivity problems, check your internet connection and restart loader.')
            input('Press enter >> ')
    
api = API('https://api.collapseloader.org/')