from threading import Thread
from time import sleep, time

from pypresence import Presence

from .Data import data
from .Settings import settings


class RPC(Thread):
    """RPC, used to display activity in Discord"""

    def __init__(self, *args, **kwargs) -> None:
        super().__init__(*args, **kwargs)

    client_id = '1225803664204234772'
    RPC = Presence(client_id)

    details = 'Choosing a client'

    start_time = time()
    disabled = True if settings.get('rpc', 'Loader') == 'False' else False

    def update(self):
        """Updates the activity"""

        try:
            self.RPC.update(state=settings.get('nickname', 'Loader'), details=self.details, large_image='https://i.imgur.com/ZpWg110.gif',
                            buttons=[
                                {'label': 'Discord', 'url': 'https://collapseloader.org/discord'},
                                {'label': 'collapseloader.org', 'url': 'https://collapseloader.org'}
                            ],
                            start=self.start_time,
                            large_text=f'Version {data.version}')
        except Exception:
            try:
                self.RPC.connect()

            except Exception:
                pass

    def run(self):
        """Starts a thread for the rpc"""

        try:
            self.RPC.connect()
        except Exception as e:
            pass
        
        while True:
            if not self.disabled:
                self.update()

            else:
                self.RPC.clear()

            sleep(5)


rpc = RPC()