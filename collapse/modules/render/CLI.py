import ctypes
import os
from time import sleep

from rich.prompt import Confirm, IntPrompt

from ...constants import SYSTEM
from ...developer import SKIP_ANIMATIONS
from ..render.Header import header
from ..storage.Data import console, data
from ..storage.Settings import settings
from ..utils.clients.Client import Client
from ..utils.clients.ClientManager import client_manager
from ..utils.Language import lang
from ..utils.Module import Module

selector_offset = len(client_manager.clients) + 11
functions = []

class Function:
    """Function for CLI class"""
    
    selector_offset = len(client_manager.clients) + 11

    def __init__(self, line: str, color: str = 'dark_cyan'):
        self.line_text = line
        self.color = color

        existing_func = next((func for func in functions if func.line_text == self.line_text), None)
        if existing_func is None:
            self.line = f'\n[{color}]{Function.selector_offset}. {line}[/]'
            functions.append(self)
            Function.selector_offset += 1
        else:
            self.line = existing_func.line

class Selector(Module):
    """Selector, used to select clients and tools, the main part of the CLI loader"""

    def __init__(self) -> None:
        super().__init__()
        self.text = self.make_text()
        self.offset = len(client_manager.clients)
        self.titles_states = {
            'default': f'CollapseLoader ({data.version})',
            'run': 'CollapseLoader >> {client}',
            'settings': lang.t('cli.titles.settings'),
            'configs': lang.t('cli.titles.configs'),
            'credits': lang.t('cli.titles.credits'),
        }
        self.custom_title = None if settings.get('custom_title') == 'None' else settings.get('custom_title')
        self.linux = True if SYSTEM == 'posix' else False
        
        self.header = header.text

        if self.offset == 0:
            self.warn(lang.t('cli.no-clients'))
            self.text += f'\n\n{lang.t('cli.no-clients')}!\n'

        self.debug(lang.t('cli.text-created'))

    def make_text(self) -> str:
        """Creates the text for the selector"""
        
        text = f'\n[bold]{lang.t('cli.menu-header')}[/]\n'
        text += '\n'.join(f'{i + 1}. {client}' for i, client in enumerate(client_manager.clients))
        text += '\n'
        
        function_lines = [
            lang.t('cli.settings-menu'),
            lang.t('cli.configs-menu'),
            lang.t('cli.select-username'),
            lang.t('cli.enter-ram'),
            lang.t('cli.ghost-mode'),
            lang.t('cli.credits-donators'),
            lang.t('cli.exit')
        ]
        
        text += ''.join(Function(line, 'dark_red' if line == 'Exit' else 'dark_cyan').line for line in function_lines)
        
        return text

    def refresh_text(self) -> None:
        """Refreshes text property"""
        self.text = self.make_text()

    def show(self) -> None:
        """Print text to screen"""
        if SKIP_ANIMATIONS or not settings.use_option('disable_animation'):
            console.print(self.text, highlight=False)
            return

        self.animate(self.text, highlight=False)

    def select(self) -> str:
        """Requires user selection"""
        return console.input(f'{lang.t('cli.select')} >> ')

    def pause(self) -> None:
        """Pauses to allow the user to read the text"""
        os.system('pause')

    def ask(self, question: str) -> bool:
        """Asks the user confirm for an action"""
        return Confirm.ask(question)
    
    def ask_int(self, question: str) -> int:
        """Asks the user for an integer"""
        return IntPrompt.ask(question)
    
    def get_client_by_index(self, index: int) -> Client:
        """Returns the client by index"""
        return client_manager.clients[index - 1]

    def select_username(self) -> str:
        """Asks for a nickname"""
        return input(f'{lang.t("cli.select-username-prompt")} >> ')

    def clear(self) -> None:
        """Clears the console"""
        os.system('cls')

    def set_title(self, text: str = f'CollapseLoader ({data.version})', title_type: str = None) -> None:
        """Sets the window title"""
        if not self.linux:
            if self.custom_title is None:
                try:
                    ctypes.windll.kernel32.SetConsoleTitleW(text if title_type is None else self.titles_states[title_type])
                except KeyError:
                    self.error(lang.t('cli.titles.cannot-find').format(title_type))
            else:
                ctypes.windll.kernel32.SetConsoleTitleW(self.custom_title)

    def reset_title(self) -> None:
        """Resets the window title"""
        self.set_title(title_type='default')

    def animate(self, text: str, highlight: bool = True) -> None:
        """Create an animated effect with a delay between each line"""
        if SKIP_ANIMATIONS or not settings.use_option('disable_animation'):
            console.print(text, highlight=highlight)
        else:
            for line in text.split('\n'):
                console.print(line, highlight=highlight)
                sleep(0.015)

    def hide_console(self):
        """Hides the console window"""
        hwnd = ctypes.windll.kernel32.GetConsoleWindow()
        
        if hwnd != 0:
            ctypes.windll.user32.ShowWindow(hwnd, 6)
            ctypes.windll.user32.ShowWindow(hwnd, 0)
            
        else:
            self.error(lang.t('cli.hide-console-error'))

    def show_console(self):
        """Shows the console window"""
        hwnd = ctypes.windll.kernel32.GetConsoleWindow()
        
        if hwnd != 0:
            ctypes.windll.user32.ShowWindow(hwnd, 5)
        else:
            self.error(lang.t('cli.show-console-error'))

selector = Selector()
