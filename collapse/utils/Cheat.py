from .Settings import settings
from .Logger import logger
from .Data import data
import os

class Cheat:
    def __init__(self, name: str, link: str, main_class: str = 'net.minecraft.client.main.Main', version: str = '1.12.2') -> None:
        self.name = name
        self.link = link
        self.filename = os.path.basename(self.link)
        self.path = data.root_dir + self.filename 
        self.path_dir = data.root_dir + os.path.splitext(self.filename)[0] + '/'
        self.jar = os.path.splitext(self.filename)[0] + '.jar'
        self.main_class = main_class
        self.version = version

    def download(self) -> True:
        """Downloading cheat files"""

        if os.path.isdir(self.path_dir):
            logger.debug(f'Client {self.name} already downloaded')
            return

        logger.debug('Downloading client')

        data.download(self.link)
    
    def run(self):
        """Run client"""

        data.download('jre-17.0.10.zip')
        data.download('libraries.zip')
        data.download('natives.zip')
        data.download('assets.zip')
        
        logger.info(f'Running client {self.name}')
  
        os.chdir('.\\' + self.path_dir)

        os.system(f"..\\jre-17.0.10\\bin\\java.exe -Xmx{settings.get('ram')}M -Djava.library.path=..\\natives; -cp ..\\libraries\*;.\\{self.jar} {self.main_class} --username {settings.get('nickname')} --gameDir {self.path_dir} --assetDir ..\\assets --assetIndex 1.12.2 --uuid N/A --accessToken 0 --userType legacy --version {self.version}")

        logger.info('Exited from minecraft')