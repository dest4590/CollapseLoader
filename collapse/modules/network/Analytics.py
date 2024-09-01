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
        try:
            r = network.get(f'{api.server}api/analytics/start', params={'version': data.version}).json()
        
            if r['status'] == 'success':
                self.debug('Successfully sent analytics request for loader start')
        
            elif r['status'] == 'error':
                self.error(f'Failed to send analytics request for loader start {r["message"]}')
        
        except Exception as e:
            self.error('Failed to send analytics request for loader start', e)

    def client_run(self, client_id: int):
        """Send a request to the analytics server when the client runs"""
        try:
            r = network.get(f'{api.server}api/analytics/client', params={'username': settings.get('nickname'), 'client_id': client_id}).json()
        
            if r['status'] == 'success':
                self.debug('Successfully sent analytics request for client run')
            
            elif r['status'] == 'error':
                self.error(f'Failed to send analytics request for client run {r["message"]}')
        
        except Exception as e:
            self.error('Failed to send analytics request for client run', e)

analytics = Analytics()