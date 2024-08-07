import time
import requests

from ..modules.Module import Module
from ..static import SERVERS
from .Network import network

class Servers(Module):
    """Class to manage and check the availability of servers"""

    def __init__(self, urls: list) -> None:
        """Initialize with a list of server URLs"""
        super().__init__()
        self.servers = urls

    def check_servers(self) -> str:
        """Check the servers for availability and return the first accessible server"""
        for server in self.servers[:]:
            try:
                start_time = time.time()
                r = network.get(f'https://{server}/')
                end_time = time.time()

                self.debug(f'Server {server} responds with {r.status_code} and {((end_time - start_time) * 1000):.2f} ms of ping')

                self.info(f'Using {server} server')
                return f'https://{server}/'
            except requests.exceptions.RequestException:
                self.info(f"The server {server} is down/inaccessible")
                self.servers.remove(server)

        return 'https://google.com/'


servers = Servers(SERVERS)
