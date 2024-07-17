from rich import print

from .CLI import console, selector
from .Logger import logger
from .Settings import settings

option_list = []

class Option:
    """The Option class represents a configurable option."""

    def __init__(self, name: str, description: str = '', option_type = str, default_value = object, callback = None, highlight: bool = False) -> None:
        self.name = name
        self.description = description
        self.option_type = option_type
        self.value = settings.get(name)
        self.default_value = default_value
        self.callback = callback
        self.highlight = highlight

        if description:
            option_list.append(self)

    @property
    def line(self) -> str:
        """Returns a formatted string representing the option"""
        self.value = settings.get(self.name)
        return f"{self.name.title().replace('_', ' ')}[/] / [light_salmon3]{self.description}[/] * [medium_purple3]{self.value}[/]"

    def create(self, value = None, header: str = 'Options') -> None:
        """Creates a new option in the settings"""
        if not settings.get(self.name, header):
            if value is not None:
                settings.set(self.name, value, header)
                logger.debug(f'Created {self} option with value: {value} ({header})')
            else:
                settings.set(self.name, self.default_value, header)
                logger.debug(f'Created {self} option with default value: {self.default_value} ({header})')
    def save(self, value: object) -> None:
        """Saves option to settings file"""
        if self.option_type == str:
            settings.set(self.name, value)
            logger.info(f'Set option {self} to {value}')

        elif self.option_type == bool:
            settings.set(self.name, value)
            logger.info(f'Switched option {self} to {value}')

        if self.callback is not None:
            self.callback()
            logger.debug('Executing callback')

    def input(self) -> None:
        """Handles user input for the option"""
        if self.option_type == str:
            new = console.input(f'Enter value for {self} (enter "RESET" to reset option): ')
            if new.upper() != 'RESET':
                self.save(new)
            else:
                self.reset()

        elif self.option_type == bool:
            current_value = settings.get(self.name)
            self.save(not current_value.lower() == 'true')

    def reset(self) -> None:
        """Reset option with default value"""
        self.save(self.default_value)
        logger.debug(f'Resetting option {self}')

    @staticmethod
    def get_option_by_index(index: int) -> 'Option':
        """Gets the option by its index"""
        return option_list[index - 1]

    def __str__(self):
        """Returns option name as title"""
        return self.name.title().replace('_', ' ')


Option('nickname', 'User nicknamse for minecraft', default_value='CollapseUser', highlight=True)
Option('custom_title', 'Changes window title for all states (None for disable)', default_value='None').create('None')
Option('hide_logo', 'Hides logo and links from main screen', bool, False).create()
Option('hide_messages', 'Hides messages from main screen', bool, False).create()
Option('disable_caching', 'Disables the caching system', bool, False).create()
Option('use_short_logo', 'Use short variant of logo', bool, False).create()
Option('hide_links', 'Hide loader social links in main menu', bool, False).create()

class Menu:
    """Options menu"""

    def __init__(self) -> None:
        self.offset = len(option_list)

    def show(self) -> None:
        """Displays the options menu"""

        selector.set_title(title_type='settings')

        while True:
            print('\n')
            option_lines = [f'[{"green" if not option.highlight else "green3"}]{i + 1}. {option.line}' for i, option in enumerate(option_list)]
            option_lines.append(f'[dark_red]{self.offset + 1}. Return[/]')
            option_lines.append(f'[bright_red]{self.offset + 2}. Reset all options[/]')
            console.print('\n'.join(option_lines), highlight=False)

            try:
                i = int(console.input('Choose option: '))

                if i <= len(option_list):
                    Option.get_option_by_index(i).input()
                elif i == self.offset + 1:
                    break
                elif i == self.offset + 2: # reset
                    if selector.ask('Are you sure you want to reset all the settings (y,n)'):
                        for option in option_list:
                            option.reset()
            except ValueError:
                logger.error('Choose a valid number')
                continue

        selector.reset_title()

options_menu = Menu()
