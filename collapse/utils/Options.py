from rich import print

from .Logger import logger
from .CLI import selector, console
from .Settings import settings

option_list = []

class Option:
    """Option"""
    def __init__(self, name: str, description: str = '', option_type = str):
        self.name = name
        self.description = description
        self.option_type = option_type
        self.value = settings.get(name, 'Options')

        if description != '':
            option_list.append(self)

    @property
    def line(self):
        self.value = settings.get(self.name, 'Options')
        return f"{self.name.title()}[/] / [violet]{self.description}[/] * [medium_purple3]{self.value}[/]"

    def create(self, value, header: str = 'Options'):
        if not settings.get(self.name, header):
            settings.set(self.name, value, header)
            logger.debug(f'Created {self.name} option with value: {value} ({header})')

    def input(self):
        if self.option_type == str:
            new = console.input(f'Enter value for {self.name}: ')

            settings.set(self.name, new, 'Options')
            logger.info(f'Set setting {self.name} to {new}')
            selector.pause()

        elif self.option_type == bool:
            current_value = settings.get(self.name, 'Options')
            new_value = not (current_value.lower() == 'true')
            settings.set(self.name, new_value, 'Options')

            logger.info(f'Switched setting {self.name}')

    @staticmethod
    def get_option_by_index(index: int):
        """gets the option through its index"""
        return option_list[index - 1]

Option('nickname', 'User nickname for minecraft')
Option('show_nickname', 'is responsible for whether nickname will be shown in RPC', bool).create(False)

class Menu:
    """Options menu"""
    def __init__(self):
        self.offset = len(option_list)

    def show(self):
        while True:
            print('\n')
            option_lines = [f'[green]{i + 1}. {option.line}' for i, option in enumerate(option_list)]
            option_lines.append(f'[dark_red]{self.offset + 1}. Return[/]')

            console.print('\n'.join(option_lines), highlight=False)

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
