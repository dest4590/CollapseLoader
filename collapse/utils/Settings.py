import configparser
import os

from .Data import data
from .Logger import logger


class Settings:
    """Settings manager, used to save user preferences"""

    def __init__(self, file: str = 'config.ini') -> None:
        self.file = file
        self.config = configparser.ConfigParser()
        self.config_path = data.get_local(self.file)

        if os.path.exists(self.config_path):
            self.config.read(self.config_path)
        else:
            logger.debug('Config file created')
            with open(self.config_path, 'w', encoding='utf-8') as cfg:
                cfg.write('')

        logger.debug('Initialized Settings')

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

    def get(self, key: str, header: str = 'Options') -> None:
        """Get a setting value"""
        if header in self.config and key in self.config[header]:
            return self.config[header][key]

        return False

    def use_option(self, name: str):
        """Сhecked whether the setting is true or false, done for convenience"""
        return settings.get(name) == 'False'


settings = Settings()
