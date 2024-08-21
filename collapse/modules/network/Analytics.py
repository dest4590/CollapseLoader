from .Network import network
from ..utils.Module import Module
from ..storage.Data import data
from ..network.API import api

class Analytics(Module):
    """Class for handling analytics"""
    
    def __init__(self):
        super().__init__()
    
    def loader_start(self):
        """Send a request to the analytics server when the loader starts"""
        network.get(f'{api.server}api/analytics/start', params={'version': data.version})
        
    def client_run(self, client_id: int):
        """Send a request to the analytics server when the client runs"""
        network.get(f'{api.server}api/analytics/client', params={'username': data.version, 'client-id': client_id})
        
analytics = Analytics()