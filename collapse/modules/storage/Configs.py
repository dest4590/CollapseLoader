import os

from rich import print

from ..utils.Module import Module
from ..network.API import api
from ..utils.Cheat import Cheat
from ..utils.Cheats import cheat_manager
from ..render.CLI import console, selector
from ..utils.Logger import logger

config_list = []

class Config:
    """A class representing a configuration object"""
    
    def __init__(self, id: int, file: str, config_path: str, server: str, client_id: int) -> None:
        self.id = id
        self.file = file
        self.cheat = Configs.get_cheat_by_id(client_id)
        self.config_path = config_path
        self.server = server
        self.server_line = f", server: [light_steel_blue]{server}[/]" if server != '-' else ''
        self.filename = os.path.basename(self.file)
        config_list.append(self)
        
    @property
    def line(self) -> str:
        is_installed = os.path.exists(f'{self.cheat.path_dir}{self.config_path}{self.filename}')
        return f"""{self.filename} {f'[green][+][/]' if is_installed else '[red][-][/]'}{self.server_line}"""

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
                    cheat.configs.append(Config(config['id'], config['file'], config['config_path'], config['server'], cheat.id))
    
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

        print('\n[bold]Configs[/]\n')

        while True:
            grouped_configs = self.group_configs_by_cheat()
            config_lines = []
            config_map = {}
            index = 1

            for cheat_name, configs in grouped_configs.items():
                config_lines.append(f'- [bold]{cheat_name}:[/]')
                for config in configs:
                    identifier = str(index) if index <= 99 else chr(87 + index)
                    config_lines.append(f'  {identifier}. {config.line}')
                    config_map[identifier] = config
                    index += 1

            config_lines.append(f'[dark_red]{index}. Return[/]')
            console.print('\n'.join(config_lines), highlight=False)

            try:
                choice = console.input('Choose config: ')

                if choice in config_map:
                    config_map[choice].cheat.load_config(config_map[choice])
                elif choice == str(index):
                    break
            except ValueError:
                logger.error('Choose a valid number or letter')
                continue

        selector.reset_title()

    def group_configs_by_cheat(self):
        """Groups configurations by cheat"""
        grouped_configs = {}
        for config in config_list:
            cheat_name = config.cheat.name
            if cheat_name not in grouped_configs:
                grouped_configs[cheat_name] = []
            grouped_configs[cheat_name].append(config)
        return grouped_configs

configs = Configs()
config_menu = ConfigMenu()