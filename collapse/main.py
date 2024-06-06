import random
import shutil
import sys
import time
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

def initialize_settings():
    """Initialize user settings with default values if not already set."""
    if not settings.get('nickname'):
        settings.set('nickname', f'Collapse{random.randint(1000, 9999)}')
        logger.debug('Nickname setup')
        logger.warn('Remember to change your nickname!')

    if not settings.get('ram'):
        settings.set('ram', 2048)
        logger.debug('Ram setup')

    if not settings.get('rpc'):
        settings.set('rpc', True)
        logger.debug('RPC setup')

def handle_rpc():
    """Handle the RPC settings and start the RPC service if necessary."""
    if '_child.py' not in sys.argv[0]:
        rpc.daemon = True
        rpc.start()

def display_main_menu():
    """Display the main menu with logo and options."""
    print('[bold white]' + logo.full)
    print('[bold green]' + logo.tagline)
    print('[italic]VER: ' + data.version)
    print('[slate_blue3]Discord: https://collapseloader.org/discord')
    print('[dodger_blue1]Telegram: https://t.me/collapseloader')
    selector.show()

def handle_selection(choosed):
    """Handle the user's menu selection."""
    if choosed <= len(cheats):
        cheat = selector.get_cheat_by_index(choosed)
        cheat.download()
        cheat.run()
    elif choosed == selector.offset + 11:
        settings.set('nickname', selector.select_username())
        logger.debug('Changed nickname')
        selector.pause()
    elif choosed == selector.offset + 12:
        settings.set('ram', selector.select_ram() * 1024)
        logger.debug('Changed ram')
        selector.pause()
    elif choosed == selector.offset + 13:
        handle_rpc_toggle()
    elif choosed == selector.offset + 14:
        logger.debug('Clean folders [y,n]')
        cheatcleaner.scan_folders()
    elif choosed == selector.offset + 15:
        handle_data_folder_removal()
    elif choosed == selector.offset + 16:
        quit()
    else:
        logger.error('Choose number')
        selector.pause()

def handle_rpc_toggle():
    """Toggle the RPC setting on or off."""
    if settings.get('rpc') == 'True':
        logger.info('Disabled RPC')
        settings.set('rpc', False)
        rpc.disabled = True
    else:
        logger.info('Enabled RPC')
        settings.set('rpc', True)
        rpc.disabled = False
        rpc.start_time = time.time()
    selector.update_text()
    selector.pause()

def handle_data_folder_removal():
    """Handle the removal of the data folder after user confirmation."""
    logger.debug('Removing data folder')
    if selector.ask('You definitely want to delete the loader data folder, this can also delete all your configs as well [y,n]'):
        shutil.rmtree('data', True)

def main():
    """Main function to run the loader."""
    updater.check_version()
    initialize_settings()
    handle_rpc()

    if '_child.py' not in sys.argv[0]:
        while True:
            display_main_menu()
            try:
                choosed = int(selector.select())
                handle_selection(choosed)
            except ValueError:
                logger.error('Choose number')
                continue
main()