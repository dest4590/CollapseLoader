from rich import print

from .Logger import logger
from .Selector import selector, console
from .Settings import settings

option_list = []

class Option:
    """Option"""
    def __init__(self, name: str, description: str = ''):
        self.name = name
        self.description = description
        self.value = settings.get(name, 'Options')

        self.line = f"{name.capitalize()}[/] / [violet]{description}[/] * [medium_purple3]{self.value}[/]"

        if description != '':
            option_list.append(self)

    def create(self, value, header: str = 'Options'):
        if not settings.get(self.name, header):
            settings.set(self.name, value, header)
            logger.debug(f'Created {self.name} option with value: {value} ({header})')

    def input(self):
        new = console.input(f'Enter value for {self.name}: ')

        settings.set(self.name, new, 'Options')
        logger.info(f'Set setting {self.name} to {new}')
        selector.pause()

    @staticmethod
    def get_option_by_index(index: int):
        """gets the option through its index"""
        return option_list[index - 1]

Option('nickname', 'User nickname for minecraft')

class Menu:
    """Options menu"""
    def __init__(self):
        self.offset = len(option_list)

    def show(self):
        while True:
            print('\n')
            console.print('\n'.join(
                [f'[green]{i + 1}. {option.line}' for i, option in enumerate(option_list)] +
                [f'[dark_red]{self.offset + 1}. Return[/]']
            ), highlight=False)

            
            try:
                i = int(console.input('Choose option: '))

                if i <= len(option_list):
                    Option.get_option_by_index(i).input()

                elif i == self.offset + 1:
                    break
            except ValueError:
                logger.error('Choose number')
                continue

options_menu = Menu()