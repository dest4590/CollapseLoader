import winreg as wrg 

from .Logger import logger

# made for the future

class Registry:
    """class to interact with the Windows registry."""

    def __init__(self):
        self.location = wrg.HKEY_CURRENT_USER
        self.soft = wrg.OpenKeyEx(self.location, r"SOFTWARE\\", 0, wrg.KEY_SET_VALUE) 
        logger.debug('Initialized Registry')

    def set_value(self, name: str, value: str, path: str):
        """sets a registry value"""
        wrg.SetValueEx(wrg.OpenKeyEx(self.location, path, 0, wrg.KEY_SET_VALUE), name, 0, wrg.REG_SZ, value)

    def edit_value(self, name: str, value: str, path: str):
        """edits an existing registry value"""
        wrg.SetValueEx(wrg.OpenKeyEx(self.location, path, 0, wrg.KEY_SET_VALUE), name, 0, wrg.REG_SZ, value)

    def remove_value(self, name: str, path: str):
        """removes a registry value"""
        wrg.DeleteValue(wrg.OpenKeyEx(self.location, path, 0, wrg.KEY_SET_VALUE), name)

    def create_key(self, subkey: str, path: str):
        """creates a new registry key under the current key"""
        wrg.CreateKeyEx(wrg.OpenKeyEx(self.location, path, 0, wrg.KEY_SET_VALUE), subkey)

    def get_value(self, name: str, path: str):
        return wrg.QueryValueEx(wrg.OpenKeyEx(self.location, path, 0, wrg.KEY_SET_VALUE), name)[0]

regedit = Registry()