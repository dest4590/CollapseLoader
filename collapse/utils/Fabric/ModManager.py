import os
from ..Logger import logger


class ModManager:
    def __init__(self, root_folder: str):
        self.root_folder = root_folder

    def get_mod_list(self):
        return os.listdir(self.root_folder)
    
    def get_mod(self, name: str):
        return self.root_folder + name
    
    def activate(self, name: str):
        if name.endswith('.disabled'):
            logger.debug(f'Enabling mod: {name}')
            os.rename(self.root_folder + f'{name}.disabled', self.root_folder + name.replace('.disabled', ''))
    
    def deactivate(self, name: str):
        if name.endswith('.jar'):
            logger.debug(f'Disabling mod: {name}')
            os.rename(self.root_folder + name, self.root_folder + f'{name}.disabled')