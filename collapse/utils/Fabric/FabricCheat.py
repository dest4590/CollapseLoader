from contextlib import chdir
from tqdm import tqdm
import requests
import time
import os

from ..Logger import logger
from ..Data import data
from ..RPC import rpc
from ..Settings import settings
from .ModManager import ModManager

mods = []

class FabricCheat:
    def __init__(self, name: str, link: str, version: str = '1.20.4', internal: bool = False) -> None:
        self.name = name
        self.link = link

        self.filename = os.path.basename(self.link)
        self.path = data.root_dir + self.filename 
        self.path_dir = data.root_dir + 'fabric-loader-0.15.9-1.20.4/'
        self.jar = os.path.splitext(self.filename)[0] + '.jar'
        
        self.main_class = 'net.fabricmc.loader.impl.launch.knot.KnotClient'
        self.version = version
        self.internal = internal
        
        self.mod_manager = ModManager(self.path_dir + 'mods/')

        mods.append(self.filename)

    def download(self) -> True:
        """Downloading cheat files"""

        if os.path.isfile(self.path_dir + self.jar):
            logger.debug(f'Client {self.name} already downloaded')
            return

        logger.debug('Downloading client')

        data.download('fabric-loader-0.15.9-1.20.4.jar')

    def download_mod(self) -> True:
        # for mod in self.mod_manager.get_mod_list():
        #     # If two and more cheats are enabled
        #     if self.filename != mod and mod in mods:
        #         self.mod_manager.deactivate(mod)

        if self.filename in self.mod_manager.get_mod_list():
            logger.debug('Mod already installed')
            return

        response = requests.get(data.server + self.filename, stream=True)
 
        total_size = int(response.headers.get('content-length', 0))

        with tqdm(total=total_size, desc=self.path_dir + 'mods/', unit="B", unit_scale=True, ascii=True, ncols=100, colour='blue') as progressbar:
            with open(self.path_dir + 'mods/' + self.filename, "wb") as f:
                for d in response.iter_content(1024):
                    f.write(d)
                    progressbar.update(len(d))

        return True
    
    def run(self):
        """Run client"""

        rpc.details = f'Running {self.name}'
        rpc.start_time = time.time()

        # Downloading requirements
        data.download('jre-21.0.2.zip')
        data.download('libraries-fabric.zip')
        data.download('natives-fabric-1.20.4.zip')
        data.download('assets-1.20.4.zip') # 600mb :(
        data.download('fabric-api-0.96.11+1.20.4.jar', mod=True)
        self.download_mod()

        
        logger.info(f'Running client {self.name}')
        with chdir('.\\' + self.path_dir):
            # Using backslash var, because f-strings not supporting it in expressions
            bc = '\\'

            command = f"..\\jre-21.0.2\\bin\\java.exe -Xverify:none -Xmx{settings.get('ram')}M -Dminecraft.launcher.version=1.6.84-j  -DFabricMcEmu=net.minecraft.client.main.Main -Dminecraft.launcher.brand=java-minecraft-launcher -Djava.library.path=..\\natives-fabric-1.20.4 -cp ..\\libraries-fabric\\*;{f'.{bc}{self.jar}' + f' {self.main_class}'} --username {settings.get('nickname')} --gameDir .\\ --assetsDir ..\\assets-1.20.4 --assetIndex 12 --uuid 3f7f52fb0c9240889ff4bd1d1a3d44d3 --accessToken null --clientId c4502edb-87c6-40cb-b595-64a280cf8906 --xuid 0 --userType msa --versionType release --version {self.version}"

            logger.debug(command)

            os.system(command)

            logger.info('Exited from minecraft')
 
            rpc.start_time = time.time()
            rpc.details = 'Choosing a client'