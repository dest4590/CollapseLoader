import ctypes
import os
from time import sleep

from ..utils.Module import Module
from ...static import SKIP_ANIMATIONS, LINUX    
from ..utils.Cheat import Cheat
from ..utils.Cheats import cheat_manager
from ..utils.Logger import logger
from ..storage.Data import console, data
from ..storage.Settings import settings

selector_offset = len(cheat_manager.cheats) + 11
functions = []

class Function:
    """Function for CLI class"""
    selector_offset = len(cheat_manager.cheats) + 11

    def __init__(self, line: str, color: str = 'dark_cyan'):
        self.line_text = line
        self.color = color

        existing_func = next((func for func in functions if func.line_text == self.line_text), None)
        if existing_func is None:
            self.line = f'\n[{color}]{Function.selector_offset}. {line}[/]'
            functions.append(self)
            Function.selector_offset += 1
        else:
            self.line = existing_func.line

class Selector(Module):
    """Selector, used to select clients and tools, the main part of the CLI loader"""

    def __init__(self) -> None:
        super().__init__()
        self.text = self.make_text()
        self.offset = len(cheat_manager.cheats)
        self.titles_states = {
            'default': f'CollapseLoader ({data.version})',
            'run': 'CollapseLoader >> {client}',
            'settings': 'CollapseLoader <Settings>',
            'configs': 'CollapseLoader <Configs>',
        }
        self.custom_title = None if settings.get('custom_title') == 'None' else settings.get('custom_title')

        if self.offset == 0:
            self.warn('No clients available')
            self.text += '\n\nNo clients available!\n'

        self.debug('Created selector text')

    def make_text(self) -> str:
        """Returns text"""
        # add title
        text = '\n[bold]CLIENTS & TOOLS[/]\n'
        # add clients
        text += '\n'.join(f'{i + 1}. {cheat}' for i, cheat in enumerate(cheat_manager.cheats))
        text += '\n'
        
        function_lines = [
            'Select username', 
            'Enter RAM',
            'Ghost mode (PANIC)', 
            'Remove data folder', 
            'Settings Menu',
            'Configs Menu [italic][turquoise4](BETA)[/]',
            'Exit'
        ]
        
        text += ''.join(Function(line, 'dark_red' if line == 'Exit' else 'dark_cyan').line for line in function_lines)
        
        return text

    def refresh_text(self) -> None:
        """Refreshes text property"""
        self.text = self.make_text()

    def show(self) -> None:
        """Print text to screen"""
        if SKIP_ANIMATIONS or not settings.use_option('disable_animation'):
            console.print(self.text, highlight=False)
            return

        self.animate(self.text, highlight=False)

    def select(self) -> str:
        """Requires user selection"""
        return console.input('Select >> ')

    def pause(self) -> None:
        """Pauses to allow the user to read the text"""
        os.system('pause')

    def ask(self, question: str) -> bool:
        """Asks the user for an action"""
        while True:
            i = console.input(f'{question} >> ').lower()
            if i in ['y', 'yes', 'да', 'н']:
                return True
            elif i in ['n', 'no', 'нет']:
                return False
            else:
                console.print('Select [bold]y[/] or [bold]n[/]')

    def get_cheat_by_index(self, index: int) -> Cheat:
        """Gets the cheat through its index"""
        return cheat_manager.cheats[index - 1]

    def select_username(self) -> str:
        """Asks for the user's nickname"""
        return input('Enter nickname >> ')

    def select_ram(self) -> int:
        """Asks how much RAM to use"""
        while True:
            try:
                return int(input('Enter ram in gigabytes >> '))
            except ValueError:
                logger.error('Enter gigabytes (2, 4, 8)')

    def clear(self) -> None:
        """Just clears text"""
        os.system('cls')

    def set_title(self, text: str = f'CollapseLoader ({data.version})', title_type: str = None) -> None:
        """Changes window title"""
        if not LINUX:
            if self.custom_title is None:
                ctypes.windll.kernel32.SetConsoleTitleW(text if title_type is None else self.titles_states[title_type])
            else:
                ctypes.windll.kernel32.SetConsoleTitleW(self.custom_title)

    def reset_title(self) -> None:
        """Sets default window title"""
        self.set_title(title_type='default')

    def animate(self, text: str, highlight: bool = True) -> None:
        """Create an animated effect with a delay between each line"""
        if SKIP_ANIMATIONS or not settings.use_option('disable_animation'):
            console.print(text, highlight=highlight)
        else:
            for line in text.split('\n'):
                console.print(line, highlight=highlight)
                sleep(0.015)

selector = Selector()
