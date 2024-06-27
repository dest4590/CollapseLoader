import os
import shutil

from .CLI import selector
from .Logger import logger


class CheatCleaner:
    """Cleans cheat folders"""
    def __init__(self):
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
            f'C:\\Users\\{os.getlogin()}\\.th-client'
        ]

        logger.debug('Initialized CheatCleaner')

    def scan_folders(self):
        """Scans all folders in array, and remove its"""
        if selector.ask('Remove all cheats folder (y,n)?\nall of your configs will be [red bold]ANNIGILATED.[/]'):
            for folder in self.folders:
                if os.path.isdir(folder):
                    logger.info('Removing folder: ' + folder)

                    shutil.rmtree(folder, ignore_errors=True)


cheatcleaner = CheatCleaner()