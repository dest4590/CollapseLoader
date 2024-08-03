import os

from rich import print

from ..modules.Module import Module
from .API import api
from .Cheat import Cheat
from .Cheats import cheat_manager
from .CLI import console, selector
from .Logger import logger

config_list = []

class Config:
    """A class representing a configuration object"""
    
    def __init__(self, id: int, file: str, config_path: str, client_id: int) -> None:
        self.id = id
        self.file = file
        self.cheat = Configs.get_cheat_by_id(client_id)
        self.config_path = config_path
        self.filename = os.path.basename(self.file)
        config_list.append(self)
        
    @property
    def line(self) -> str:
        is_installed = os.path.exists(f'{self.cheat.path_dir}{self.config_path}{self.filename}')
        return f'{self.cheat.name} - {self.filename} [green][Installed][/]' if is_installed else f'{self.cheat.name} - {self.filename} [red][Not installed][/]'

class Configs(Module):
    """A class representing configurations"""

    def __init__(self):
        super().__init__()
        self.configs = api.get('configs').json()
        self.init_configs()
        
    def init_configs(self):
        """Initialize configurations"""
        
        self.debug(f'Found {len(self.configs)} configs')
        
        for config in self.configs:
            client_id = config['client'].split('/')[-2]

            for cheat in cheat_manager.cheats:
                if cheat.id == int(client_id):
                    cheat.configs.append(Config(config['id'], config['file'], config['config_path'], cheat.id))
    
    @staticmethod
    def get_cheat_by_id(client_id: int) -> Cheat:
        for cheat in cheat_manager.cheats:
            if cheat.id == int(client_id):
                return cheat

class ConfigMenu:
    """Configurations menu"""

    def __init__(self) -> None:
        self.offset = len(config_list)

    def show(self) -> None:
        """Displays the configurations menu"""

        selector.set_title(title_type='configs')

        while True:
            print('\n')
            config_lines = [f'[green]{i + 1}. {config.line}' for i, config in enumerate(config_list)]
            config_lines.append(f'[dark_red]{self.offset + 1}. Return[/]')
            console.print('\n'.join(config_lines), highlight=False)

            try:
                i = int(console.input('Choose config: '))

                if i <= len(config_list):
                    config_list[i - 1].cheat.load_config(config_list[i - 1])
                elif i == self.offset + 1:
                    break
            except ValueError:
                logger.error('Choose a valid number')
                continue

        selector.reset_title()

configs = Configs()
config_menu = ConfigMenu()