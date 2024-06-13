import os
import shutil

from .Logger import logger
from .CLI import selector


class CheatCleaner:
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
            'C:\\Excellent'
        ]

    def scan_folders(self):
        if selector.ask('Remove all cheats folder \[y,n]?\nall of your configs will be [red bold]ANNIGILATED.[/]'):
            for folder in self.folders:
                if os.path.isdir(folder):
                    logger.info('Removing folder: ' + folder)

                    shutil.rmtree(folder, ignore_errors=True)

cheatcleaner = CheatCleaner()