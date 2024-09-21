import os
import shutil
from glob import glob

from .logger import logger


class Builder:
    """Class to build to .exe file"""

    def __init__(self, name: str = 'CollapseLoader', icon: str = 'logo.ico', dev: bool = False) -> None:
        self.name = name
        
        if dev:
            logger.info('--dev flag detected, building dev version')
            self.name += '_dev'
        
        self.icon = icon

    def build(self, remove_build: bool = True):
        """Starts the build"""
        
        logger.info('Building CollapseLoader, this may take a while...')
        
        # Remove old .exe builds
        for file in glob('*.exe'):
            os.remove(file)
        
        os.system(f'''pyinstaller --onefile --clean --console --name "{self.name}" --icon "collapse\\assets\\{self.icon}" run.py''')

        # Move .exe file to root
        for file in glob('dist/*.exe'):
            shutil.move(file, './')

        # Remove build folder
        if remove_build:
            self.clear_all()

    def clear_all(self):
        """Deletes all unnecessary files"""
        logger.info('Clearing build files')
        
        if os.path.exists('build/'):
            shutil.rmtree('build/')

        if os.path.exists('dist/'):
            shutil.rmtree('dist/')

        # Remove spec file
        for spec in glob('*.spec'):
            os.remove(spec)