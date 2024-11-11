import os

from ..storage.Data import data
from ..utils.Language import lang
from ..utils.Module import Module


class ModManager(Module):
    """Manages mods in the specified root folder"""

    def __init__(self, root_folder: str) -> None:
        """Initialize ModManager with the given root folder"""
        super().__init__()
        self.root_folder = root_folder
        self.mods_folder = os.path.join(self.root_folder, "mods")

    def get_mod_list(self) -> list:
        """Get a list of mods in the root folder"""
        return os.listdir(self.root_folder)

    def get_mod(self, name: str) -> str:
        """Get the full path of the specified mod"""
        return os.path.join(self.root_folder, name)

    def activate(self, name: str) -> None:
        """Activate a disabled mod"""
        if name.endswith(".disabled"):
            self.debug(lang.t("modmanager.activate").format(name))
            os.rename(self.get_mod(name), self.get_mod(name.replace(".disabled", "")))

    def deactivate(self, name: str) -> None:
        """Deactivate an enabled mod"""
        if name.endswith(".jar"):
            self.debug(lang.t("modmanager.deactivate").format(name))
            os.rename(self.get_mod(name), self.get_mod(f"{name}.disabled"))

    def install(self, mod: str) -> None:
        """Install mod by name in the mods folder"""
        if not os.path.exists(self.mods_folder):
            os.makedirs(self.mods_folder)

        if not os.path.exists(f"{self.mods_folder}/{mod}"):
            self.info(lang.t("modmanager.install").format(mod))
            data.download(mod, f"{self.mods_folder}/{mod}", True)
