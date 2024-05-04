from rich.console import Console
import os

from .Logger import logger
from .Cheats import cheats
from .Cheat import Cheat

console = Console()

class Function:
    def __init__(self, line: str, number: int, color: str = 'dark_cyan'):
        self.line = f'\n[{color}]{number}. {line}[/]'

class Selector:
    """Selector, used to select clients, and tools, the main part of the CLI loader"""

    def __init__(self) -> None:
        """class init that creates a text variable"""

        logger.debug('Initialized selector')
        
        self.text = '\n[bold]CHEATS & TOOLS[/]\n'
        self.text += '\n'.join([f'{i + 1}. {cheat.name}' for i, cheat in enumerate(cheats)])
        self.text += '\n'
        self.text += ''.join([Function('Select username', 19).line,
                                Function('Enter RAM', 20).line,
                                Function('Enable/Disable RPC', 21).line,
                                Function('Ghost mode (PANIC)', 22).line,
                                Function('Remove data folder', 23).line,
                                Function('Exit', 24, 'dark_red').line])

        logger.debug('Created self.text')

    def show(self) -> None:
        """print self.text to screen"""
        console.print(self.text, highlight=False)

    def select(self) -> str:
        """requires user selection"""
        return input('Select >> ')
    
    def pause(self) -> None:
        """pauses to allow the user to read the text"""
        input('Press enter >> ')

    def ask(self, question: str) -> bool:
        """asks the user for an action"""
        while True:
            i = input(f'{question} >> ')
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
                logger.error('Enter gigabytes (2, 4)')

    def clear(self) -> None:
        os.system('cls')

selector = Selector()