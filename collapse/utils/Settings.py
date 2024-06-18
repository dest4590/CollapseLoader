import configparser
import os

from .Data import data
from .Logger import logger


class Settings:
    """Settings manager, used to save user preferences"""

    def __init__(self, file: str = 'config.ini'):
        self.file = file
        self.config = configparser.ConfigParser()
        self.config_path = data.get_local('config.ini')

        if os.path.exists(self.config_path):
            self.config.read(self.config_path)

        else:
            logger.debug('Сonfig file created')
            with open(self.config_path, 'w') as cfg:
                cfg.write('')

        logger.debug('Initialized Settings')

    def save(self):
        """save config to file"""
        with open(self.config_path, 'w') as cfg:
            self.config.write(cfg)

    def set(self, key: str, value: str, header: str = 'Loader'):
        """sets setting, and saves it to the config."""

        if header not in self.config:
            self.config[header] = {}

        value_as_string = str(value)

        self.config[header][key] = value_as_string
        self.save()

    def get(self, key: str, header: str = 'Loader'):
        """get setting"""

        try:
            return self.config[header][key]
        except KeyError:
            return False


settings = Settings()