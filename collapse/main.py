import random
import shutil
import sys

from rich import print
from rich.progress import BarColumn, Progress, SpinnerColumn, TextColumn

with Progress(SpinnerColumn(), TextColumn("[progress.description]{task.description}"), BarColumn(), transient=True) as progress:
    loading_task = progress.add_task("[blue]Loading modules", total=None)

    from .utils.CheatCleaner import cheatcleaner
    from .utils.Cheats import cheat_manager
    from .utils.CLI import selector
    from .utils.Data import data
    from .utils.Logger import logger
    from .utils.Logo import logo
    from .utils.Message import messageclient
    from .utils.Options import Option, options_menu
    from .utils.Settings import settings
    from .utils.Updater import updater

def initialize_settings():
    """Initialize user settings with default values if not already set."""
    Option('nickname').create(f'Collapse{random.randint(1000, 9999)}')

    if not settings.get('nickname', 'Options'):
        logger.warn('Remember to change your nickname!')

    Option('ram').create(2048, 'Loader')
    Option('rpc').create(True, 'Loader')
    Option('read_messages').create('0,', 'Loader')

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
    if choosed <= len(cheat_manager.cheats):
        cheat = selector.get_cheat_by_index(choosed)
        cheat.download()
        cheat.run()
    elif choosed == selector.offset + 11: # Select username
        settings.set('nickname', selector.select_username(), 'Options')
        logger.debug('Changed nickname')
        selector.pause()
    elif choosed == selector.offset + 12: # Enter RAM
        settings.set('ram', selector.select_ram() * 1024)
        logger.debug('Changed ram')
        selector.pause()
    elif choosed == selector.offset + 13: # Ghost mode (PANIC)
        logger.debug('Clean folders (y,n)')
        cheatcleaner.scan_folders()
    elif choosed == selector.offset + 14: # Remove data folder
        handle_data_folder_removal()
    elif choosed == selector.offset + 15: # Settings Menu
        options_menu.show()
    elif choosed == selector.offset + 16: # Exit
        sys.exit(1)
    else:
        logger.error('Choose number')
        selector.pause()

def handle_data_folder_removal():
    """Handle the removal of the data folder after user confirmation."""
    logger.debug('Removing data folder')
    if selector.ask('You definitely want to delete the loader data folder, this can also delete all your configs as well (y,n)'):
        shutil.rmtree('data', True)

        logger.info('Removed data folder')

def handle_message_showing():
    """Handle the showing of messages."""

    messageclient.show_messages()

def main():
    """Main function to run the loader."""
    updater.check_version()
    initialize_settings()
    selector.set_title('CollapseLoader')

    if '_child.py' not in sys.argv[0]:
        while True:
            display_main_menu()

            if not messageclient.shown:
                handle_message_showing()

            try:
                choosed = int(selector.select())
                handle_selection(choosed)
            except ValueError:
                logger.error('Choose number')
                continue
