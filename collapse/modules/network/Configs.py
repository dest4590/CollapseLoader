import os

from rich import print

from ..render.CLI import console, selector
from ..utils.clients.Client import Client
from ..utils.clients.ClientManager import client_manager
from ..utils.Logger import logger
from ..utils.Module import Module
from .API import api

config_list = []

class Config:
    """A class representing a configuration object"""
    def __init__(self, id: int, file: str, config_path: str, server: str, client_id: int) -> None:
        self.id = id
        self.file = file
        self.client = Configs.get_client_by_id(client_id)
        self.config_path = config_path
        self.server = server
        self.server_line = f", server: [light_steel_blue]{server}[/]" if server != '-' else ''
        self.filename = os.path.basename(self.file)
        config_list.append(self)
        
    @property
    def line(self) -> str:
        is_installed = os.path.exists(f'{self.client.path_dir}{self.config_path}{self.filename}')
        return f"""{self.filename} {f'[green][+][/]' if is_installed else '[red][-][/]'}{self.server_line}"""

class Configs(Module):
    """A class representing configurations"""

    def __init__(self):
        super().__init__()
        self.configs = api.get('configs')

        if self.configs is not None:
            self.configs = self.configs.json()
            self.init_configs()

    def init_configs(self):
        """Initialize configurations"""
        
        self.debug(f'Found {len(self.configs)} configs')
        
        for config in self.configs:
            client_name = config['client_name']

            for client in client_manager.clients:
                if client.name in client_name:
                    client.configs.append(Config(config['id'], config['file'], config['config_path'], config['server'], client.id))
    
    @staticmethod
    def get_client_by_id(client_id: int) -> Client:
        for client in client_manager.clients:
            if client.id == int(client_id):
                return client

class ConfigMenu:
    """Configurations menu"""

    def __init__(self) -> None:
        self.offset = len(config_list)

    def show(self) -> None:
        """Displays the configurations menu"""

        selector.set_title(title_type='configs')

        print('\n[bold]Configs[/]\n')

        while True:
            grouped_configs = self.group_configs_by_client()
            config_lines = []
            config_map = {}
            index = 1

            for client_name, configs in grouped_configs.items():
                config_lines.append(f'- [bold]{client_name}:[/]')
                for config in configs:
                    identifier = str(index) if index <= 99 else chr(87 + index)
                    config_lines.append(f'  {identifier}. {config.line}')
                    config_map[identifier] = config
                    index += 1

            config_lines.append(f'\n[dark_cyan]{index + 1}. Install all configs[/]')
            config_lines.append(f'[dark_red]{index + 2}. Return[/]')
            console.print('\n'.join(config_lines), highlight=False)

            try:
                choice = selector.ask_int('Choose config')
                if str(choice) in config_map:
                    config_map[str(choice)].client.load_config(config_map[str(choice)])
                elif choice == index + 1:
                    for config in config_list:
                        config.client.load_config(config)
                elif choice == index + 2:
                    break
            except ValueError:
                logger.error('Choose a valid number or letter')
                continue

        selector.reset_title()

    def group_configs_by_client(self):
        """Groups configurations by client"""
        grouped_configs = {}
        for config in config_list:
            client_name = config.client.name
            if client_name not in grouped_configs:
                grouped_configs[client_name] = []
            grouped_configs[client_name].append(config)
        return grouped_configs

configs = Configs()
config_menu = ConfigMenu()