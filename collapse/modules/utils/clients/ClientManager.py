import os

from ....developer import SHOW_HIDDEN_CLIENTS
from ...network.API import api
from ...storage.Cache import cache
from ...storage.Data import data
from ...storage.Settings import settings
from ...utils.Language import lang
from ..Module import Module
from .Client import Client
from .FabricClient import FabricClient


class ClientManager(Module):
    """Class to manage and load clients from the API"""

    def __init__(self) -> None:
        super().__init__()
        self.clients: list[Client] = []
        self._load_clients()

    def _load_clients(self) -> list:
        """Load clients from the API and return a list of client instances"""
        clients = api.get('clients')
        fabric_clients = api.get('fabric_clients')
        
        if clients is None and fabric_clients is None:
            self.error(lang.t('clientmanager.error'))
            if not os.path.exists(cache.path):
                self.error(lang.t('cache.client-cache-not-found'))

            else:
                c = cache.get()
                creation_time = c['_meta']['creation_time']
                self.info(lang.t('cache.using-last-cache').format(creation_time))

                self.make_array(c['clients'])
                
            return

        all_clients: dict = clients.json() + fabric_clients.json()

        clients = []

        if clients is not None:
            cache.save(all_clients)
            self.make_array(all_clients)

        return all_clients
    
    def client_line(self, client: dict) -> str:
        """Returns a formatted string representing the client"""
        version = f" <{client['version']}>" if not settings.use_option('show_client_version') else ''
        return f"{client['name']}{version}"
    
    def make_array(self, clients: dict) -> None:
        """Adds clients to array"""
        for client in clients:
            if not client['fabric']:
                if client["show_in_loader"] or SHOW_HIDDEN_CLIENTS:
                    self.clients.append(
                        Client(
                            name=self.client_line(client),
                            link=data.get_url(client["filename"]),
                            main_class=client["main_class"],
                            version=client["version"],
                            internal=client["internal"],
                            working=client["working"],
                            id=client["id"],
                            fabric=client["fabric"]
                        )
                    )
            else:
                if client["show_in_loader"] or SHOW_HIDDEN_CLIENTS:
                    self.clients.append(
                        FabricClient(
                            name=self.client_line(client),
                            link=data.get_url(client["filename"]),
                            version=client["version"],
                            working=client["working"],
                            id=client["id"],
                            fabric=client["fabric"]
                        )
                    )
                
    def refresh(self) -> None:
        """Refresh clients"""
        self.clients: list[Client] = []
        self._load_clients()
        
    def get_client_by_name(self, name: str) -> Client:
        """Get client by name"""
        for client in self.clients:
            if client.name.lower() in name.lower():
                return client

client_manager = ClientManager()
