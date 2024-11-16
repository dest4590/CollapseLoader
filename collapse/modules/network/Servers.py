import requests

from ..utils.Language import lang
from ..utils.Module import Module
from .Network import NameResolutionError, network


class CDNServer:
    def __init__(self, url: str) -> None:
        self.url = url

    def check(self) -> bool:
        try:
            r = network.get(f"https://{self.url}/")
            return r.status_code == 200
        except requests.exceptions.RequestException:
            return False
        except NameResolutionError:
            return False


class WebServer(CDNServer):
    def __init__(self, url: str) -> None:
        super().__init__(url)


SERVERS = [
    CDNServer("cdn.collapseloader.org"),
    CDNServer("cdn-ru.collapseloader.org"),
    CDNServer("cdncollapse.ttfdk.lol"),
]

WEB_SERVERS = [
    WebServer("web.collapseloader.org"),
    WebServer("web2.collapseloader.org"),
    WebServer("webcollapse.ttfdk.lol"),
]


class Servers(Module):
    """Class to manage and check the availability of servers"""

    def __init__(self) -> None:
        """Initialize with a list of server URLs"""
        super().__init__()
        self.servers = SERVERS
        self.web_servers = WEB_SERVERS

        self.cdn_server = ""
        self.web_server = ""

    def check_servers(self) -> None:
        """Check all servers for availability and return the first accessible server"""

        for server in self.servers[:]:
            if server.check():
                self.debug(lang.t("servers.server-respond").format(server.url, 200))

                self.cdn_server = f"https://{server.url}/"
                self.info(lang.t("servers.use-server").format("CDN", server.url))

                break
            else:
                self.info(lang.t("servers.server-not-accessible").format(server.url))
                self.servers.remove(server)

        for server in self.web_servers[:]:
            if server.check():
                self.debug(lang.t("servers.server-respond").format(server.url, 200))

                self.web_server = f"https://{server.url}/"
                self.info(lang.t("servers.use-server").format("WEB", server.url))

                break
            else:
                self.info(lang.t("servers.server-not-accessible").format(server.url))
                self.web_servers.remove(server)


servers = Servers()
