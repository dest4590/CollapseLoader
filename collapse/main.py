import random
import shutil
import sys

from rich.progress import BarColumn, Progress, SpinnerColumn, TextColumn

from .modules.utils.Fixes import console

with Progress(SpinnerColumn(), TextColumn("[progress.description]{task.description}"), BarColumn(), transient=True, console=console) as progress:
    loading_task = progress.add_task("[blue]Loading modules", total=None)

    from .modules.utils.Cheats import cheat_manager
    from .modules.render.CLI import selector
    from .modules.utils.Logger import logger
    from .modules.utils.Logo import logo
    from .modules.network.Message import messageclient
    from .modules.network.Updater import updater
    from .modules.storage.Options import Option, options_menu
    from .modules.storage.CheatCleaner import cheatcleaner
    from .modules.network.Configs import config_menu
    from .modules.storage.Data import data
    from .modules.storage.Settings import settings

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
    if choosed <= len(cheat_manager.cheats):
        cheat = selector.get_cheat_by_index(choosed)
        cheat.run()
    elif choosed == selector.offset + 11: # Select username
        settings.set('nickname', selector.select_username())
        logger.debug('Changed nickname')
        selector.pause()
    elif choosed == selector.offset + 12: # Enter RAM
        settings.set('ram', selector.select_ram() * 1024, 'Loader')
        logger.debug('Changed ram')
        selector.pause()
    elif choosed == selector.offset + 13: # Ghost mode (PANIC)
        cheatcleaner.scan_folders()
    elif choosed == selector.offset + 14: # Remove data folder
        handle_data_folder_removal()
    elif choosed == selector.offset + 15: # Settings Menu
        options_menu.show()
    elif choosed == selector.offset + 16: # Configs Menu
        config_menu.show()
    elif choosed == selector.offset + 17: # Exit
        sys.exit(1)
    else:
        logger.error('Choose number')
        selector.pause()

def handle_data_folder_removal() -> None:
    """Handle the removal of the data folder after user confirmation"""

    logger.debug('Removing data folder')
    if selector.ask('You definitely want to delete the loader data folder, this can also delete all your configs as well (y,n)'):
        shutil.rmtree('data', True)

        logger.info('Removed data folder')

def main() -> None:
    """Main function to run the loader"""
    updater.check_version()
    initialize_settings()
    selector.set_title(selector.titles_states['default'])

    if '_child.py' not in sys.argv[0]:
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