from pypresence import Presence
from threading import Thread
from time import sleep, time

from .Data import data
from .Settings import settings

class RPC(Thread):
    client_id = '1225803664204234772'
    RPC = Presence(client_id)

    details = 'Choosing a client'

    start_time = time()
    disabled = True if settings.get('rpc') == 'False' else False

    def update(self):
        self.RPC.update(state=settings.get('nickname'), details=self.details, large_image=data.server_assets + 'rpc.gif', 
                        buttons=[
                            {'label': 'Discord', 'url': 'https://collapseloader.org/discord'},
                            {'label': 'collapseloader.org', 'url': 'https://collapseloader.org'} 
                        ],
                        start=self.start_time,
                        large_text=f'Version {data.version}' )

    def run(self):
        try:
            self.RPC.connect()
        except:
            return
        
        while True:
            if not self.disabled:
                self.update()

            else:
                self.RPC.clear()

            sleep(1)

rpc = RPC()