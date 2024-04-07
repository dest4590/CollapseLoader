from rich import print
from time import time
import random

from .utils.Logo import logo
from .utils.Selector import selector
from .utils.Cheats import cheats
from .utils.Logger import logger
from .utils.Settings import settings
from .utils.RPC import rpc
from .utils.Data import data

# Using rich library for displaying bold and color texts
print('[bold white]' + logo.full)
print('[bold green]' + logo.tagline)
print('[italic]VER: ' + data.version)

rpc.start()

if not settings.get('nickname'):
    settings.set('nickname', f'CollapseUser{random.randint(10000, 99999)}')
    logger.debug('Nickname setup')
    print('[bold gray23]Remember to change your nickname!')

if not settings.get('ram'):
    settings.set('ram', 2048)
    logger.debug('Ram setup')

if not settings.get('rpc'):
    settings.set('rpc', True)
    logger.debug('RPC setup')

while True:
    selector.show()

    try:
        choosed = int(selector.select())
        
    except ValueError:
        logger.error('Choose number')
        pass

    if choosed <= len(cheats):
        cheat = selector.get_cheat_by_index(choosed)
        cheat.download()
        cheat.run()

    elif choosed == 20:
        settings.set('nickname', selector.select_username())
        logger.debug('Changed nickname')

    elif choosed == 21:
        settings.set('ram', selector.select_ram() * 1024)
        logger.debug('Changed ram')

    elif choosed == 22:
        if settings.get('rpc') == 'True':
            # Disable
            logger.info('Disabled RPC')
            settings.set('rpc', False)
            rpc.disabled = True
            selector.info()
        
        elif settings.get('rpc') == 'False':
            # Enable
            logger.info('Enabled RPC')
            settings.set('rpc', True)
            rpc.disabled = False
            rpc.start_time = time()
            selector.info()
            
            
    else:
        logger.error('Choose number')