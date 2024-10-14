import os
from contextlib import chdir
from datetime import datetime
from subprocess import PIPE, STDOUT, Popen
from threading import Thread
from time import sleep

from rich.progress import BarColumn, Progress, SpinnerColumn, TextColumn

from ...network.Analytics import analytics
from ...storage.Data import console, data
from ...storage.ModManager import ModManager
from ...storage.Settings import settings
from ...utils.Language import lang
from ..LogChecker import logchecker
from ..RPC import rpc
from .Client import Client


def update_time(task_id, progress, start_time) -> None:
    """Updates the time for the progress bar by ticking one second"""
    while True:
        elapsed_time = datetime.now() - start_time
        progress.update(
            task_id,
            time=str(elapsed_time).split('.', maxsplit=1)[0])

        sleep(1)


class FabricClient(Client):
    """Client class for running clients"""
    
    def __init__(self, name: str, link: str, version: str = '1.12.2', working: bool = True, id: int = 1, fabric: bool = False) -> None:
        super().__init__(name, link, version, working, id, fabric=True)
        
        self.mod_manager = ModManager(self.path_dir)
        
    def download(self) -> True:
        """Downloading client files"""
        jar_file = 'fabric-1.21.jar'
        destination = os.path.join(self.path_dir, jar_file)

        if os.path.isfile(destination):
            self.debug(lang.t('clients.already-downloaded-fabric'))
            return
        else:
            os.makedirs(self.path_dir, exist_ok=True)
            self.info(lang.t('clients.downloading-fabric'))

        data.download(jar_file, destination, True)
        
    def run(self) -> None:
        """Run client"""
        from ...render.CLI import selector

        selector.set_title(selector.titles_states['run'].format(client=self.name))
        
        rpc.details = lang.t('rpc.playing').format(self.name)

        # Downloading requirements
        data.download('jre-21.0.2.zip')

        self.info(lang.t('clients.downloading-fabric'))
        data.download('libraries-1.21.zip')
        data.download('natives.zip')

        data.download('assets.zip')
        
        self.download()
        
        self.debug('Installing mod')
        
        self.mod_manager.install('thunderhack-1.7.jar')
        self.mod_manager.install('fabric-api-0.102.0+1.21.jar')

        self.info(lang.t('clients.running-fabric').format(self.name))

        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(pulse_style='gray'),
            TextColumn("{task.fields[session]} {task.fields[time]}"),
            transient=True, console=console
        ) as progress:
            start_time = datetime.now()
            task_id = progress.add_task(
                lang.t('clients.progress.task').format(self.name, settings.get('nickname')),
                session=lang.t('clients.progress.session'),
                time="00:00:00",
                total=None
            )

            Thread(target=update_time, args=(task_id, progress, start_time), daemon=True).start()

            with chdir('.\\' + self.path_dir):
                # Using backslash var, because f-strings not supporting it in expressions
                bc = '\\'

                path_sep = ';' if os.name == 'nt' else ':'

                libraries_dir = 'libraries-1.21'
                natives_dir = 'natives'
                assets_dir = 'assets'

                classpath = f'..{bc}{libraries_dir}{bc}*'
                native_path = f'..{bc}{natives_dir};'

                asset_path = f'.{bc}{assets_dir}' if self.internal and os.path.isdir(assets_dir) else f'..{bc}{assets_dir}'

                java_command = [
                    f"..\\jre-21.0.2\\bin\\java{'w' if self.silent else ''}.exe",
                    "-Xverify:none",
                    f"-Xmx{settings.get('ram')}M",
                    f"-Djava.library.path={native_path}",
                    f"-cp {classpath}{path_sep}.{bc}fabric-1.21.jar net.fabricmc.loader.impl.launch.knot.KnotClient",
                    f"--username {settings.get('nickname')}",
                    "--gameDir .\\",
                    f"--assetsDir {asset_path}",
                    f"--assetIndex 1.16",
                    "--uuid 00000000-0000-0000-0000-000000000000",
                    "--accessToken 0",
                    "--userType legacy",
                    f"--version {self.version}"
                ]

                command = ' '.join(java_command)
                self.debug(command)
                
                analytics.client_run(self.id)

                process = Popen(command, stdout=PIPE, stderr=STDOUT)
                buffer = []

                for line in process.stdout:
                    _ = line.decode('utf-8', errors='ignore')
                    progress.print(_, end='', markup=False, highlight=False)
                    buffer.append(_)

                logchecker.check_logs(buffer)
 
                self.info(lang.t('clients.finished'))

        # Return default title
        selector.reset_title()

        rpc.details = rpc.default_details
        rpc.update()