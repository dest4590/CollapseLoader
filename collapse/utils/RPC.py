from pypresence import Presence
from threading import Thread
from time import sleep, time

from .Data import data

class RPC(Thread):
    client_id = '1225803664204234772'
    RPC = Presence(client_id)

    details = 'Picks a cheat'
    state = f'Version {data.version}' 

    start_time = time()

    def update(self):
        self.RPC.update(state=self.state, details=self.details, large_image=data.server_assets + 'rpc.gif', 
                        buttons=[
                            {'label': 'collapseloader.org', 'url': 'https://collapseloader.org'}, 
                            {'label': 'Discord', 'url': 'https://collapseloader.org/discord'}
                        ],
                        start=self.start_time)

    def run(self):
        try:
            self.RPC.connect()
        except:
            return
        
        while True:
            self.update()
            sleep(10)

rpc = RPC()