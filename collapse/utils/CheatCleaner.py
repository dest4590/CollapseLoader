import os
import shutil

from ..modules.Module import Module
from .CLI import selector


class CheatCleaner(Module):
    """Cleans cheat folders"""
    def __init__(self) -> None:
        super().__init__()
        if not selector.linux:
            # Absolute path
            self.folders = [
                'C:\\Celestial',
                'C:\\baritone',
                'C:\\shaderpacks',
                'C:\\resourcepacks',
                'C:\\RockAntiLeak',
                'C:\\Rockstar',
                'C:\\MoonProject',
                'C:\\hachrecode',
                'C:\\Nursultan',
                f'C:\\Users\\{os.getlogin()}\\.avalon',
                'C:\\Excellent',
                f'C:\\Users\\{os.getlogin()}\\AppData\\Roaming\\.antiautistleak',
                f'C:\\Users\\{os.getlogin()}\\.th-client',
                'C:\\RichRecode'
            ]

    def scan_folders(self) -> None:
        """Scans all folders in array, and remove its"""
        if selector.ask('Remove all cheats folder (y,n)?\nall of your configs will be [red bold]ANNIGILATED.[/]'):
            for folder in self.folders:
                if os.path.isdir(folder):
                    self.info('Removing folder: ' + folder)
                    shutil.rmtree(folder, ignore_errors=True)

        selector.pause()

cheatcleaner = CheatCleaner()
