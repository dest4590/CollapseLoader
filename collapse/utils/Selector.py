from rich.console import Console
import os

from .Logger import logger
from .Cheats import cheats
from .Cheat import Cheat

console = Console()

class Selector:
    """Selector, used to select clients, and tools, the main part of the CLI loader"""

    def __init__(self) -> None:
        logger.debug('Initialized selector')
        
        self.text = ''
        self.text += '\n[bold]CHEATS & TOOLS[/]\n'

        for cheat in enumerate(cheats, 1):
            # EXM: 1. Celestial
            self.text += f'{cheat[0]}. {cheat[1].name}\n'

        self.text += '\n[dark_cyan]19. Select username[/]'
        self.text += '\n[dark_cyan]20. Enter RAM[/]'
        self.text += '\n[dark_cyan]21. Enable/Disable RPC[/]'
        self.text += '\n[dark_cyan]22. Ghost mode (PANIC)[/]'
        self.text += '\n[dark_red]23. Exit[/]'

    def show(self) -> None:    
        console.print(self.text, highlight=False)

    def select(self) -> str:
        return input('Select >> ')
    
    def pause(self) -> None:
        input('Press enter >> ')

    def ask(self, question: str) -> str:
        """asks the user for an action"""

        while True:
            i = input(f'{question} >> ')

            if i in ['y', '', 'yes', 'да', 'н']:
                return True
            
            elif i in ['n', 'т', 'нет']:
                return False
            
            else:
                pass
    
    def get_cheat_by_index(self, index: int) -> Cheat:
        for i, c in enumerate(cheats, 1):
            if i == index:
                return c
            
    def select_username(self) -> str:
        return input('Enter nickname >> ')

    def select_ram(self) -> int:
        while True:
            try:
                return int(input('Enter ram in gigabytes >> '))
            
            except ValueError:
                logger.error('Enter gigabytes (2, 4)')

    def clear(self) -> None:
        os.system('cls')

selector = Selector()
