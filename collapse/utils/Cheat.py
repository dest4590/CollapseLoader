from contextlib import chdir
import time
import os

from .Data import data
from .Logger import logger
from .Settings import settings
from .RPC import rpc


class Cheat:
    def __init__(self, name: str, link: str, main_class: str = 'net.minecraft.client.main.Main', version: str = '1.12.2', internal: bool = False) -> None:
        self.name = name
        self.link = link

        self.filename = os.path.basename(self.link)
        self.path = data.root_dir + self.filename 
        self.path_dir = data.root_dir + os.path.splitext(self.filename)[0] + '/'
        self.jar = os.path.splitext(self.filename)[0] + '.jar'
        
        self.main_class = main_class
        self.version = version
        self.internal = internal
        self.silent = False

    def download(self) -> True:
        """Downloading cheat files"""

        if os.path.isfile(self.path_dir + self.jar):
            logger.debug(f'Client {self.name} already downloaded')
            return

        logger.info('Downloading client')

        data.download(self.link)
    
    def run(self):
        """Run client"""

        rpc.details = f'Running {self.name}'
        rpc.start_time = time.time()

        # Downloading requirements
        data.download('jre-21.0.2.zip')
        data.download('libraries.zip')
        data.download('natives.zip')
        data.download('assets.zip')
        
        logger.info(f'Running client {self.name}')
        with chdir('.\\' + self.path_dir):
            # Using backslash var, because f-strings not supporting it in expressions
            bc = '\\'

            command = f"..\\jre-21.0.2\\bin\\java{'w' if self.silent else ''}.exe -Xverify:none -Xmx{settings.get('ram')}M -Djava.library.path={f'.{bc}natives;' if self.internal and os.path.isdir(f'natives') else f'..{bc}natives;'} {(f'-cp .{bc}libraries{bc}*' if self.internal and os.path.isdir(f'libraries') else f'-cp ..{bc}libraries{bc}*') + ';'}{f'.{bc}' + self.jar + f' {self.main_class}'} --username {settings.get('nickname')} --gameDir .\\ --assetsDir {f'.{bc}assets' if self.internal and os.path.isdir(f'assets') else f'..{bc}assets'} --assetIndex {self.version} --uuid N/A --accessToken 0 --userType legacy --version {self.version}"

            logger.debug(command)

            os.system(command)

            logger.info('Exited from minecraft')
 
            rpc.start_time = time.time()
            rpc.details = 'Picks a cheat'