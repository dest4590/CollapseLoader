from rich import print
import random

from .utils.Logo import logo
from .utils.Selector import selector
from .utils.Cheats import cheats
from .utils.Logger import logger
from .utils.Settings import settings


# Using rich library for displaying bold and color texts
print('[bold white]' + logo.full)
print('[bold green]' + logo.tagline)

if not settings.get('nickname'):
    settings.set('nickname', f'CollapseUser{random.randint(10000, 99999)}')
    logger.debug('Nickname setup')
    print('[bold gray23]Remember to change your nickname!')

if not settings.get('ram'):
    settings.set('ram', 2048)
    logger.debug('Ram setup')

while True:
    selector.show()

    try:
        choosed = int(selector.select())
        
    except ValueError:
        logger.error('Choose number')

    if choosed <= len(cheats):
        cheat = selector.get_cheat_by_index(choosed)
        cheat.download()
        cheat.run()

    elif choosed == 8:
        settings.set('nickname', selector.select_username())
        logger.debug('Changed nickname')

    elif choosed == 9:
        settings.set('ram', selector.select_ram() * 1024)
        logger.debug('Changed ram')
            
    else:
        logger.error('Choose')