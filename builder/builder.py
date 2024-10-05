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
        
        for file in glob('*.exe'):
            os.remove(file)
    
        data_files = []
        lang_dir = os.path.join("collapse", "assets", "lang")

        for filename in os.listdir(lang_dir):
            if filename.endswith(".yml"):
                data_files.append((os.path.join(lang_dir, filename), os.path.join("collapse", "assets", "lang")))

        data_files_string = " ".join([f"--add-data \"{src};{dst}\"" for src, dst in data_files])

        command = f'pyinstaller --onefile --clean --console --name "{self.name}" --icon "collapse/assets/{self.icon}" {data_files_string} run.py'  # use f-string properly

        logger.info(f'Running command: {command}')

        os.system(command)

        for file in glob('dist/*.exe'):
            shutil.move(file, './')

        if remove_build:
            self.clear_all()

    def clear_all(self):
        """Deletes all unnecessary files"""
        logger.info('Clearing build files')
        
        if os.path.exists('build/'):
            shutil.rmtree('build/')

        if os.path.exists('dist/'):
            shutil.rmtree('dist/')

        for spec in glob('*.spec'):
            os.remove(spec)