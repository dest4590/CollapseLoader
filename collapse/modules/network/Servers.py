import requests

from ...constants import SERVERS, WEB_SERVERS
from ..utils.Language import lang
from ..utils.Module import Module
from .Network import network


class Servers(Module):
    """Class to manage and check the availability of servers"""

    def __init__(self) -> None:
        """Initialize with a list of server URLs"""
        super().__init__()
        self.servers = SERVERS
        self.web_servers = WEB_SERVERS

    def check_servers(self) -> str:
        """Check the servers for availability and return the first accessible server"""
        for server in self.servers[:]:
            try:
                r = network.get(f"https://{server}/")
                self.debug(
                    lang.t("servers.server-respond").format(server, r.status_code)
                )

                self.debug(lang.t("servers.use-server").format(server))
                return f"https://{server}/"
            except requests.exceptions.RequestException:
                self.info(lang.t("servers.server-not-accessible").format(server))
                self.servers.remove(server)

        return "https://google.com/"

    def check_web_servers(self) -> str:
        """Check the web servers for availability and return the first accessible web server"""
        for web_server in self.web_servers[:]:
            try:
                r = network.get(f"https://{web_server}/")
                self.debug(
                    lang.t("servers.server-respond").format(web_server, r.status_code)
                )

                self.debug(lang.t("servers.use-server").format(web_server))
                return f"https://{web_server}/"
            except requests.exceptions.RequestException:
                self.info(lang.t("servers.server-not-accessible").format(web_server))
                self.web_servers.remove(web_server)

        return "https://google.com/"


servers = Servers()
