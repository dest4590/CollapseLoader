import logging
import random
import sys

from .arguments import args
from .modules.utils.Logger import logger

if args.v:
    logger.setLevel(logging.DEBUG)

from .modules.storage.Data import data # isort: skip
from .modules.network.Analytics import analytics
from .modules.network.Configs import config_menu
from .modules.network.Message import messageclient
from .modules.network.Updater import updater
from .modules.render.CLI import selector
from .modules.sdk.SdkServer import server
from .modules.storage.ClientCleaner import clientcleaner
from .modules.storage.Options import Option, options_menu
from .modules.storage.Settings import settings
from .modules.utils.ClientManager import client_manager
from .modules.utils.CreditsMenu import credits_menu
from .modules.utils.Logo import logo
from .modules.utils.RPC import rpc


def initialize_settings() -> None:
    """Initialize user settings with default values if not already set"""
    Option('nickname').create(f'Collapse{random.randint(1000, 9999)}')

    if not settings.get('nickname'):
        logger.warn('Remember to change your nickname!')

    Option('ram').create(2048, 'Loader')
    Option('rpc').create(True, 'Loader')
    Option('read_messages').create('0,', 'Loader')

def display_main_menu() -> None:
    """Display the main menu with logo and options"""
    text = ''

    if settings.use_option('hide_logo'):
        logo_type = logo.full if settings.use_option('use_short_logo') else logo.short
        selector.animate(f'[bold white]{logo_type}\n', highlight=False)

        text += f'\t [italic]Version: {data.version}[/] ([steel_blue3]{data.codename.upper()}[/])\n'
        text += f'[bold green]{logo.tagline}[/]\n'

        if settings.use_option('hide_links'):
            text += '[slate_blue3]Discord: https://collapseloader.org/discord\n'
            text += '[dodger_blue1]Telegram: https://collapseloader.org/telegram'

    selector.animate(text)
    selector.show()

def handle_selection(choosed) -> None:
    """Handle the user's menu selection"""
    if choosed <= len(client_manager.clients):
        client = selector.get_client_by_index(choosed)
        client.run()
    elif choosed == selector.offset + 11:
        options_menu.show()
    elif choosed == selector.offset + 12:
        config_menu.show()
    elif choosed == selector.offset + 13:
        settings.set('nickname', selector.select_username())
        logger.debug('Changed nickname')
    elif choosed == selector.offset + 14:
        settings.set('ram', selector.ask_int('Select ram (in gigabytes)') * 1024, 'Loader')
        logger.debug('Changed ram')
    elif choosed == selector.offset + 15:
        clientcleaner.scan_folders()
    elif choosed == selector.offset + 16:
        credits_menu.show()
    elif choosed == selector.offset + 17: # Exit
        sys.exit(1)
    else:
        logger.error('Choose number')
        selector.pause()

def main() -> None:
    """Main function to run the loader"""
    if '_child.py' not in sys.argv[0]:
        initialize_settings()
        
        updater.check_version()
        analytics.loader_start()
        
        if args.server:
            server.run()
    
        else:
            selector.set_title(selector.titles_states['default'])
            
            rpc.start()
        
            while True:
                display_main_menu()

                if not messageclient.shown:
                    messageclient.show_messages()

                try:
                    choosed = int(selector.select())
                    handle_selection(choosed)
                    
                except ValueError:
                    logger.error('Choose number')
                    continue
