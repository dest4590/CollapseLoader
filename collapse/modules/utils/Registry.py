import winreg as wrg

# made for the future


class Registry:
    """Class to interact with the Windows registry"""

    def __init__(self):
        self.location = wrg.HKEY_CURRENT_USER

    def delete_value(self, path: str) -> None:
        """Deletes all values under the specified path"""
        key = wrg.OpenKey(self.location, path, 0, wrg.KEY_ALL_ACCESS)

        for _ in range(0, wrg.QueryInfoKey(key)[1]):
            value_name = wrg.EnumValue(key, 0)[0]
            wrg.DeleteValue(key, value_name)

        wrg.CloseKey(key)

    def delete_key(self, path: str) -> None:
        """Deletes the specified key and all its subkeys"""
        key = wrg.OpenKey(self.location, path, 0, wrg.KEY_ALL_ACCESS)
        self._delete_subkeys(key)
        wrg.DeleteKey(self.location, path)

    def _delete_subkeys(self, key) -> None:
        """Helper function to delete all subkeys under a given key"""
        while True:
            try:
                subkey_name = wrg.EnumKey(key, 0)
                subkey = wrg.OpenKey(key, subkey_name, 0, wrg.KEY_ALL_ACCESS)
                self._delete_subkeys(subkey)
                wrg.DeleteKey(key, subkey_name)

            except OSError:
                break


regedit = Registry()
