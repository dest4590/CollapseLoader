import os

from .Logger import logger


class ModManager:
    """Manages mods in the specified root folder"""

    def __init__(self, root_folder: str) -> None:
        """Initialize ModManager with the given root folder"""
        self.root_folder = root_folder

    def get_mod_list(self) -> list:
        """Get a list of mods in the root folder"""
        return os.listdir(self.root_folder)

    def get_mod(self, name: str) -> str:
        """Get the full path of the specified mod"""
        return os.path.join(self.root_folder, name)

    def activate(self, name: str) -> None:
        """Activate a disabled mod"""
        if name.endswith('.disabled'):
            logger.debug(f'Enabling mod: {name}')
            os.rename(self.get_mod(name), self.get_mod(name.replace('.disabled', '')))

    def deactivate(self, name: str) -> None:
        """Deactivate an enabled mod"""
        if name.endswith('.jar'):
            logger.debug(f'Disabling mod: {name}')
            os.rename(self.get_mod(name), self.get_mod(f'{name}.disabled'))
