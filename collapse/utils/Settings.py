import configparser
import os

from .Data import data

class Settings:
    def __init__(self, file: str = 'config.ini'):
        self.file = file
        self.config = configparser.ConfigParser()
        self.config_path = data.get_local('config.ini')

        if os.path.exists(self.config_path):
            self.config.read(self.config_path)

        else:
            with open(self.config_path, 'w') as cfg:
                cfg.write('')

    def save(self):
        with open(self.config_path, 'w') as cfg:
            self.config.write(cfg)

    def set(self, key: str, value: str, header: str = 'Loader'):
        if header not in self.config:
            self.config[header] = {}

        value_as_string = str(value)

        self.config[header][key] = value_as_string
        self.save()

    def get(self, key: str, header: str = 'Loader'):
        try:
            return self.config[header][key]
        except KeyError:
            return False

settings = Settings()