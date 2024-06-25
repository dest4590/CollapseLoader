import os

from rich.console import Console

from .Cheat import Cheat
from .Cheats import cheat_manager
from .Logger import logger
from .RPC import rpc

console = Console()
selector_offset = len(cheat_manager.cheats) + 11
functions = []

class Function:
    def __init__(self, line: str, color: str = 'dark_cyan'):
        global selector_offset
        self.line_text = line

        existing_func = next((func for func in functions if func.line_text == self.line_text), None)
        if existing_func is None:
            self.line = f'\n[{color}]{selector_offset}. {line}[/]'
            functions.append(self)
            selector_offset += 1
        else:
            self.line = existing_func.line

class Selector:
    """Selector, used to select clients and tools, the main part of the CLI loader"""

    def __init__(self) -> None:
        logger.debug('Initialized Selector')
        self.text = self.make_text()
        self.offset = len(cheat_manager.cheats)

        if self.offset == 0:
            logger.warn('No clients available')
            self.text += '\n\nNo clients available, make sure you have internet\n'

        logger.debug('Created selector text')

    def make_text(self) -> str:
        text = '\n[bold]CLIENTS & TOOLS[/]\n'
        text += '\n'.join(f'{i + 1}. {cheat}' for i, cheat in enumerate(cheat_manager.cheats))
        text += '\n'
        function_lines = [
            'Select username', 
            'Enter RAM',
            'Ghost mode (PANIC)', 
            'Remove data folder', 
            'Settings Menu', 
            'Exit'
        ]
        text += ''.join(Function(line, 'dark_red' if line == 'Exit' else 'dark_cyan').line for line in function_lines)
        return text
    
    def update_text(self) -> None:
        """Refreshes text property"""
        self.text = self.make_text()

    def show(self) -> None:
        """Print text to screen"""
        console.print(self.text, highlight=False)

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
        os.system('cls')

selector = Selector()