from ..render.CLI import console, selector
from .Logger import logger


class CreditsMenu:
    """Credits and donators menu"""

    def __init__(self) -> None:
        self.text = '''
[bold]Credits & Donators[/]

< [bold]Credits[/] >
Thanks to the following people and projects for their contributions:
- [underline]Collapse Team[/] - For creating CollapseLoader

< [bold]Donators[/] >
Thanks to the following people for their donations:
- [underline yellow]leizark1338 ★[/] - For donating 10$
- [underline]dana56[/] - For donating 1$

< [bold]Special Thanks[/] > 
Thanks to the following people for their contributions:
- [underline]ttfdk[/] - For server configuration and maintenance
- [underline]Leizark[/] - For moderating the Discord server

And you, for using CollapseLoader ❤
'''

    def show(self) -> None:
        """Displays the configurations menu"""

        selector.set_title(title_type='credits')
        
        while True:
            console.print(self.text)

            if selector.ask('Quit'):
                break

        selector.reset_title()

credits_menu = CreditsMenu()