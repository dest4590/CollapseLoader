from zipfile import ZipFile
from ..logger import logger
import os

class Parser:
    def __init__(self, input_file: str) -> None:
        self.input_file = input_file
        self.minecraft = False

        logger.debug('Initialized parser')


    def scan(self):
        if not self.input_file.endswith('.jar'):
            logger.error('File is not minecraft executable.')
            return
        
        if not os.path.exists(self.input_file):
            logger.error('File does not exists.')
            return
        
        logger.info(f'Starting analyse of {self.input_file}')

        # https://stackoverflow.com/questions/8005300/opening-a-java-jar-file-from-python
        zf = ZipFile(self.input_file, 'r')

        folders = []

        try:
            file_list = zf.infolist()

            for file in file_list:
                if file.is_dir():
                    folders.append(file.filename)

            if 'net/minecraft/client/main/' in folders:
                self.minecraft = True
                logger.info('Valid minecraft client.')

        finally:
            zf.close()

        logger.info(f'Is minecraft: {self.minecraft}')