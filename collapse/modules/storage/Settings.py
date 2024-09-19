import configparser
import os

from ..utils.Module import Module
from .Data import data


class Settings(Module):
    """Settings manager, used to save user preferences"""

    def __init__(self, file: str = 'config.ini') -> None:
        super().__init__()
        self.file = file
        self.config = configparser.ConfigParser()
        self.config_path = data.get_local(self.file)

        if os.path.exists(self.config_path):
            self.config.read(self.config_path)
        else:
            self.debug('Config file created')
            with open(self.config_path, 'w', encoding='utf-8') as cfg:
                cfg.write('')

    def save(self) -> None:
        """Save config to file"""
        with open(self.config_path, 'w', encoding='utf-8') as cfg:
            self.config.write(cfg)

    def set(self, key: str, value: str, header: str = 'Options') -> None:
        """Set a setting and save it to the config"""
        if header not in self.config:
            self.config[header] = {}

        self.config[header][key] = str(value)
        self.save()

    def get(self, key: str, header: str = 'Options') -> str:
        """Get a setting value"""
        return self.config.get(header, key, fallback=None)

    def use_option(self, name: str) -> bool:
        """Check whether the setting is true or false, done for convenience"""
        if self.get(name) is None:
            return True
        
        return self.get(name) == 'False'

settings = Settings()