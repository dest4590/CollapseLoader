import requests
import time

from .Logger import logger
from ..static import SERVERS

class Servers:
    """Class to manage and check the availability of servers."""

    def __init__(self, urls: list) -> None:
        """Initialize with a list of server URLs and create a requests session"""
        self.servers = urls
        self.session = requests.Session()

    def check_servers(self) -> str:
        """Check the servers for availability and return the first accessible server"""
        for server in self.servers[:]:
            try:
                start_time = time.time()
                r = self.session.get(f'https://{server}/', timeout=5)
                end_time = time.time()
                
                logger.debug(f'Server {server} responds with {r.status_code} and {((end_time - start_time) * 1000):.2f} ms of ping')

                logger.info(f'Using {server} server')
                return f'https://{server}/'
            except requests.exceptions.RequestException:
                logger.info(f"The server {server} is down/inaccessible")
                self.servers.remove(server)

        return None


servers = Servers(SERVERS)