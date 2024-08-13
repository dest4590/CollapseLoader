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
        self._is_dirty = False  # Flag to track if config has been modified

        if os.path.exists(self.config_path):
            self.config.read(self.config_path)
        else:
            self.debug('Config file created')
            with open(self.config_path, 'w', encoding='utf-8') as cfg:
                cfg.write('')

    def save(self) -> None:
        """Save config to file only if it has been modified"""
        if self._is_dirty:
            with open(self.config_path, 'w', encoding='utf-8') as cfg:
                self.config.write(cfg)
            self._is_dirty = False

    def set(self, key: str, value: str, header: str = 'Options') -> None:
        """Set a setting and mark config as modified"""
        if header not in self.config:
            self.config[header] = {}
        self.config[header][key] = str(value)
        self._is_dirty = True

    def get(self, key: str, header: str = 'Options') -> str:
        """Get a setting value from the cached config"""
        return self.config.get(header, key, fallback=False)

    def use_option(self, name: str) -> bool:
        """Check whether the setting is true or false, done for convenience"""
        return self.get(name) == 'False'

    def __del__(self):
        """Ensure config is saved when the object is destroyed"""
        self.save()

settings = Settings()