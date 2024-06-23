from threading import Thread
from time import sleep, time

from pypresence import Presence

from .Data import data
from .Logger import logger
from .Settings import settings


class RPC(Thread):
    """RPC, used to display activity in Discord"""

    def __init__(self, *args, **kwargs) -> None:
        logger.debug('Initialized Discord RPC')
        super().__init__(*args, **kwargs)

    client_id = '1225803664204234772'
    RPC = Presence(client_id)

    details = 'Choosing a client'

    start_time = time()
    disabled = True if settings.get('rpc') == 'False' else False

    def update(self):
        """Updates the activity"""

        try:
            self.RPC.update(state=settings.get('nickname'), details=self.details, large_image='https://i.imgur.com/ZpWg110.gif',
                            buttons=[
                                {'label': 'Discord', 'url': 'https://collapseloader.org/discord'},
                                {'label': 'collapseloader.org', 'url': 'https://collapseloader.org'}
                            ],
                            start=self.start_time,
                            large_text=f'Version {data.version}')
        except Exception:
            logger.debug('RPC crashed')
            logger.debug('Trying to connect')

            try:
                self.RPC.connect()
                logger.debug('Connected to discord')

            except Exception:
                logger.debug('Cannot reconnect to Discord')


    def run(self):
        """Starts a thread for the rpc"""

        try:
            self.RPC.connect()
        except Exception as e:
            logger.debug(f'RPC error: {e}')
        
        while True:
            if not self.disabled:
                self.update()

            else:
                self.RPC.clear()

            sleep(5)


rpc = RPC()