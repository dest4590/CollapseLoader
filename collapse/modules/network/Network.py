import requests

from ...arguments import args
from ..utils.Language import lang
from ..utils.Module import Module


class NameResolutionError(Exception):
    pass


class Network(Module):
    """Module for network operations"""

    def __init__(self):
        super().__init__()

        self.session = requests.Session()
        self.timeout = args.timeout if args.timeout else 5

        if args.timeout:
            self.debug(lang.t("network.timeout").format(self.timeout))

    def get(self, url, params=None, headers=None, stream=False) -> requests.Response:
        """Make a GET request to the given URL"""
        try:
            response = self.session.get(
                url,
                params=params,
                headers=headers,
                stream=stream,
                timeout=self.timeout,
                # verify=False,
            )
            return response
        except requests.exceptions.RequestException as e:
            if "NameResolutionError" in str(e):
                raise NameResolutionError

            self.error(lang.t("network.error").format(e))
            raise e

    def close(self):
        self.session.close()


network = Network()
