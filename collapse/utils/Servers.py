import requests

from .Logger import logger
from ..static import SERVERS

class Servers:
    def __init__(self, urls: list):
        self.servers = urls
        self.session = requests.Session()

    def check_servers(self):
        for server in self.servers[:]:
            try:
                self.session.get(f'https://{server}/', timeout=5)
                logger.info(f'Using {server} server')
                return f'https://{server}/'
            except requests.exceptions.RequestException:
                logger.info(f"The server {server} is down/inaccessible")
                self.servers.remove(server)


servers = Servers(SERVERS)