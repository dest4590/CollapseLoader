from ...arguments import args, enabled_args
from ..network.API import api
from ..storage.Data import data
from ..storage.Settings import settings
from ..utils.Language import lang
from ..utils.Module import Module
from .Network import network


class Analytics(Module):
    """Class for handling analytics"""
    
    def __init__(self):
        super().__init__()
        self.disabled = False
        
        if args.disable_analytics:
            self.disabled = True
            self.info(lang.t('analytics.disabled'))
    
    def loader_start(self):
        """Send a request to the analytics server when the loader starts"""
        if self.disabled:
            return
        
        try:
            r = network.get(f'{api.server}api/analytics/start', params={'version': data.version, 'enabled_args': str(enabled_args)}).json()
        
            if r['status'] == 'success':
                self.debug(lang.t('analytics.successfuly-sent-loader'))
        
            elif r['status'] == 'error':
                self.error(lang.t('analytics.error-sent-loader').format(r["message"]))
        
        except Exception as e:
            self.error(lang.t('analytics.error-sent-loader').format(e))

    def client_run(self, client_id: int):
        """Send a request to the analytics server when the client runs"""
        if self.disabled:
            return
        
        try:
            r = network.get(f'{api.server}api/analytics/client', params={'username': settings.get('nickname'), 'client_id': client_id}).json()
        
            if r['status'] == 'success':
                self.debug(lang.t('analytics.successfuly-sent-client'))
        
            elif r['status'] == 'error':
                self.error(lang.t('analytics.error-sent-client').format(r["message"]))
        
        except Exception as e:
            self.error(lang.t('analytics.error-sent-client').format(e))

analytics = Analytics()
