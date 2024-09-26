from ..network.API import api
from ..render.CLI import console, selector


class CreditsMenu:
    """Credits and donators menu"""

    def __init__(self) -> None:
        self.text = None

    def show(self) -> None:
        """Displays the configurations menu"""
        
        if self.text is None:
            self.text = api.get('credits/', prefix=False).text
        
        selector.set_title(title_type='credits')
        
        while True:
            console.print(self.text)

            if selector.ask('Quit'):
                break

        selector.reset_title()

credits_menu = CreditsMenu()