from ..network.API import api
from ..storage.Data import data
from ..storage.Settings import settings
from ..utils.Module import Module
from .Network import network


class Analytics(Module):
    """Class for handling analytics"""
    
    def __init__(self):
        super().__init__()
    
    def loader_start(self):
        """Send a request to the analytics server when the loader starts"""
        self.debug('Send analytics request for loader start')
        network.get(f'{api.server}api/analytics/start', params={'version': data.version})
        
    def client_run(self, client_id: int):
        """Send a request to the analytics server when the client runs"""
        self.debug('Send analytics request for client run')
        network.get(f'{api.server}api/analytics/client', params={'username': settings.get('nickname'), 'client_id': client_id})

analytics = Analytics()