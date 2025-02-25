import os
import shutil

from ..render.CLI import selector
from ..utils.Language import lang
from ..utils.Module import Module


class ClientCleaner(Module):
    """Cleans client folders"""

    def __init__(self) -> None:
        super().__init__()
        if not selector.linux:
            self.login = os.getlogin()
            self.folders = [
                f"C:\\Users\\{self.login}\\AppData\\Roaming\\.antiautistleak",
                f"C:\\Users\\{self.login}\\.th-client",
                f"C:\\Users\\{self.login}\\.avalon",
                "C:\\shaderpacks",
                "C:\\Rockstar",
                "C:\\RockAntiLeak",
                "C:\\RichRecode",
                "C:\\resourcepacks",
                "C:\\Nursultan",
                "C:\\MoonProject",
                "C:\\hachrecode",
                "C:\\Excellent",
                "C:\\Celestial",
                "C:\\baritone",
                "C:\\hachrecode",
                "C:\\Nevernight",
            ]

    def scan_folders(self) -> None:
        """Scans all folders in array, and remove its"""
        if selector.ask(lang.t("clientcleaner.ask")):
            for folder in self.folders:
                if os.path.isdir(folder):
                    self.info(lang.t("clientcleaner.removing").format(folder))
                    shutil.rmtree(folder, ignore_errors=True)
        selector.pause()


clientcleaner = ClientCleaner()
