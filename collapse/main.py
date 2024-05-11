import time
import os
import random
import shutil
import sys

from rich import print

from .utils.CheatCleaner import cheatcleaner
from .utils.Cheats import cheats
from .utils.Data import data
from .utils.Logger import logger
from .utils.Logo import logo
from .utils.RPC import rpc
from .utils.Selector import selector
from .utils.Settings import settings
from .utils.Updater import updater

updater.check_version()

# Settings setup
if not settings.get('nickname'):
    settings.set('nickname', f'Collapse{random.randint(1000, 9999)}')
    logger.debug('Nickname setup')
    print('[bold gray23]Remember to change your nickname!')

if not settings.get('ram'):
    settings.set('ram', 2048)
    logger.debug('Ram setup')

if not settings.get('SafeDirName'):
    settings.set('SafeDirName', "Telеgram Desktоp") # e and o changed to cyrilic alternatives for save original folder and looks legit idk
    logger.debug('SafeDirName setup')

if not settings.get('rpc'):
    settings.set('rpc', True)
    logger.debug('RPC setup')

if not '_child.py' in sys.argv[0]:
    rpc.daemon = True
    rpc.start()

# Main thread
if not '_child.py' in sys.argv[0]:
    while True:
        # selector.clear()

        print('[bold white]' + logo.full)
        print('[bold green]' + logo.tagline)
        print('[italic]VER: ' + data.version)
        print('[blue]Discord: https://collapseloader.org/discord')
        
        selector.show()

        try:
            choosed = int(selector.select())
            
        except ValueError:
            logger.error('Choose number')
            continue

        if choosed <= len(cheats):
            cheat = selector.get_cheat_by_index(choosed)
            cheat.download()
            cheat.run()

        elif choosed == 21:
            settings.set('nickname', selector.select_username())
            logger.debug('Changed nickname')
            selector.pause()

        elif choosed == 22:
            settings.set('ram', selector.select_ram() * 1024)
            logger.debug('Changed ram')
            selector.pause()

        elif choosed == 23:
            if settings.get('rpc') == 'True':
                logger.info('Disabled RPC')
                settings.set('rpc', False)
                rpc.disabled = True
                selector.update_text()
                selector.pause()
            
            elif settings.get('rpc') == 'False':
                logger.info('Enabled RPC')
                settings.set('rpc', True)
                rpc.disabled = False
                rpc.start_time = time()
                selector.update_text()
                selector.pause()

        elif choosed == 24:
            logger.info('Clean folders [y,n]')
            cheatcleaner.scan_folders()
        
        elif choosed == 25:
            settings.set('SafeDirName', selector.select_safedirname())
            logger.debug('Changed safe directory name')
            selector.pause()

        elif choosed == 26:
            logger.info('Removing data folder')
            if selector.ask('You definitely want to delete the loader data folder, this can also delete all your configs as well [y,n]'):
                SafeDir = settings.get('SafeDirName')                
                os.rename('data', SafeDir)
                time.sleep(5)
                shutil.rmtree(SafeDir, True)

        elif choosed == 27:
            quit()

        else:
            logger.error('Choose number')
            selector.pause()
