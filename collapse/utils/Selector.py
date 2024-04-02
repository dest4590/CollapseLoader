from ..logger import logger
from rich.console import Console
from .Cheat import cheat_list
from . import Cheats

console = Console()

class Selector:
    """Selector, used to select clients, and tools, the main part of the CLI loader"""

    def __init__(self):
        logger.debug('Initialized selector.')
        
        self.indexer = enumerate(cheat_list, 1)
    
    def show(self):
        text = ''
        
        text += '\n[bold]CHEATS[/]\n'
        
        for cheat in self.indexer:
            # EXM: 1. Celestial
            text += f'{cheat[0]}. {cheat[1].name}'

        console.print(text, highlight=False)

    def select(self):
        return input('Select >> ')

selector = Selector()
