import os

from rich.console import Console

from .Cheat import Cheat
from .Cheats import cheats
from .Logger import logger
from .RPC import rpc

console = Console()

selector_offset = len(cheats) + 11
functions = []

class Function:
    def __init__(self, line: str, color: str = 'dark_cyan'):
        global selector_offset

        self.line_text = line

        if not any(func.line_text == self.line_text for func in functions):
            self.line = f'\n[{color}]{selector_offset}. {line}[/]'
            functions.append(self)
            selector_offset += 1
        else:
            self.line = next(func.line for func in functions if func.line_text == self.line_text)

class Selector:
    """Selector, used to select clients, and tools, the main part of the CLI loader"""

    def __init__(self) -> None:
        """class init that creates a text variable"""

        logger.debug('Initialized Selector')
        
        self.text = self.make_text()
        self.offset = len(cheats)

        if self.offset == 0:
            logger.warn('No clients available')
            self.text += '\n\nNo clients available, make sure you have internet\n'

        logger.debug('Created selector text')

    def make_text(self) -> str:
        text = '\n[bold]CLIENTS & TOOLS[/]\n'


        text += '\n'.join([f'{i + 1}. {cheat}' for i, cheat in enumerate(cheats)])

        text += '\n'
        text += ''.join([Function('Select username').line,
                                Function('Enter RAM').line,
                                Function('Discord RPC ' + ('[green][+][/]' if not rpc.disabled else '[red][-][/]')).line,
                                Function('Ghost mode (PANIC)').line,
                                Function('Remove data folder').line,
                                Function('Settings Menu').line,
                                Function('Exit', 'dark_red').line])
        
        return text
    
    def update_text(self) -> None:
        """refreshes text property"""
        self.text = self.make_text()

    def show(self) -> None:
        """print text to screen"""
        console.print(self.text, highlight=False)

    def select(self) -> str:
        """requires user selection"""
        return console.input('Select >> ')
    
    def pause(self) -> None:
        """pauses to allow the user to read the text"""
        os.system('pause')

    def ask(self, question: str) -> bool:
        """asks the user for an action"""
        while True:
            i = console.input(f'{question} >> ').lower()
            if i in ['y', 'yes', 'да', 'н']:
                return True
            elif i in ['n', 'no', 'нет']:
                return False
    
    def get_cheat_by_index(self, index: int) -> Cheat:
        """gets the cheat through its index"""
        return cheats[index - 1]
            
    def select_username(self) -> str:
        """asks for the user's nickname"""
        return input('Enter nickname >> ')

    def select_ram(self) -> int:
        """asks how much RAM to use"""
        while True:
            try:
                return int(input('Enter ram in gigabytes >> '))
            except ValueError:
                logger.error('Enter gigabytes (2, 4, 8)')

    def clear(self) -> None:
        os.system('cls')


selector = Selector()