import os
import random
from datetime import datetime
from subprocess import PIPE, STDOUT, Popen
from threading import Thread
from time import sleep

from os import chdir
from rich.color import ANSI_COLOR_NAMES
from rich.progress import BarColumn, Progress, SpinnerColumn, TextColumn

from ..storage.Data import console, data
from ..storage.Settings import settings
from ...static import LINUX
from .LogChecker import logchecker
from .Module import Module

class cd:
    """Context manager for changing the current working directory"""
    def __init__(self, newPath):
        self.newPath = os.path.expanduser(newPath)

    def __enter__(self):
        self.savedPath = os.getcwd()
        os.chdir(self.newPath)

    def __exit__(self, etype, value, traceback):
        os.chdir(self.savedPath)

def update_time(task_id, progress: Progress, start_time) -> None:
    """Updates the time for the progress bar by ticking one second"""
    while True:
        elapsed_time = datetime.now() - start_time
        progress.update(
            task_id,
            time=str(elapsed_time).split('.', maxsplit=1)[0])

        sleep(1)


class Cheat(Module):
    """Cheat class for running clients"""

    def __init__(self, name: str, link: str,
                 main_class: str = 'net.minecraft.client.main.Main',
                 version: str = '1.12.2',
                 category: str = 'HVH', internal: bool = False, id: int = 1) -> None:
        super().__init__()
    
        self.name = name
        self.link = link
        self.category = category
        self.id = id
        self.configs = []

        self.filename = os.path.basename(self.link)
        self.path = data.root_dir + self.filename
        self.path_dir = data.root_dir + os.path.splitext(self.filename)[0] + '/'
        self.jar = os.path.splitext(self.filename)[0] + '.jar'

        self.main_class = main_class
        
        self.cut_version = True
        self.version = version[:-2] if self.cut_version else version
        
        self.internal = internal
        self.silent = False

    def __str__(self) -> str:
        return self.name

    def download(self) -> True:
        """Downloading cheat files"""

        if os.path.isfile(self.path_dir + self.jar):
            self.debug(f'Client {self.name} already downloaded')
            return

        self.info('Downloading client')

        data.download(self.filename)
        
    def load_config(self, config) -> None:
        """Load configurations for the client"""
        
        if not os.path.exists(self.path_dir + config.config_path):
            os.makedirs(self.path_dir + config.config_path)

        data.download(config.file, f'{self.path_dir}{config.config_path}{config.filename}')
 
    def run(self) -> None:
        """Run client"""

        self.download()

        from ..render.CLI import selector

        selector.set_title(selector.titles_states['run'].format(client=self.name))

        # Downloading requirements
        if not LINUX:
            self.debug('Downloading Windows JRE')
            data.download('jre-21.0.2.zip')
        else:
            self.debug('Downloading Linux JRE')
            data.download('jre-21.0.2-linux.zip')
            
            os.system(f'chmod +x {data.root_dir}jre-21.0.2-linux/bin/java')

        if self.version.startswith('1.12'):
            self.debug('Downloading 1.12.2 libraries & natives')
            data.download('libraries-1.12.zip')
            if not LINUX:
                data.download('natives-1.12.zip')
            else:
                data.download('natives-linux.zip')

        else:
            self.debug('Downloading 1.12.2+ libraries & natives')
            data.download('libraries.zip')
            if not LINUX:
                data.download('natives.zip')
            else:
                data.download('natives-linux.zip')

        data.download('assets.zip')

        self.info(f'Running client {self.name}')
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(pulse_style=random.choice(list(ANSI_COLOR_NAMES.keys()))),
            TextColumn("[progress.description]{task.fields[session]} {task.fields[time]}"),
            transient=True, console=console
        ) as progress:
            start_time = datetime.now()
            task_id = progress.add_task(
                f"[green]Running client[/] [light_slate_blue]{self.name}[/] [light_salmon1]<{settings.get('nickname')}>[/]",
                session="[purple3]active session[/]",
                time="00:00:00",
                total=None
            )

            Thread(target=update_time, args=(task_id, progress, start_time), daemon=True).start()

            # Using backslash var, because f-strings not supporting it in expressions
            bc = '\\'
            
            with cd(os.path.join(self.path_dir)):
                path_sep = ';' if os.name == 'nt' else ':'
                
                libraries_dir = 'libraries-1.12' if self.version.startswith('1.12') else 'libraries'
                natives_dir = ('natives-1.12' if not LINUX else 'natives-linux') if self.version.startswith('1.12') else ('natives' if not LINUX else 'natives-linux')
                assets_dir = 'assets'

                if self.internal and os.path.isdir(libraries_dir):
                    classpath = f'.{bc}{libraries_dir}{bc}*'
                else:
                    classpath = f'..{bc}{libraries_dir}{bc}*'

                if self.internal and os.path.isdir(natives_dir):
                    native_path = f'.{bc}{natives_dir}{";" if not LINUX else ""}'
                else:
                    native_path = f'..{bc}{natives_dir}{";" if not LINUX else ""}'

                asset_path = f'.{bc}{assets_dir}' if self.internal and os.path.isdir(assets_dir) else f'..{bc}{assets_dir}'

                java_command = [
                    f"""{f"..{bc}jre-21.0.2" if not LINUX else f'..{bc}jre-21.0.2-linux'}\\bin\\java{'w' if self.silent else ''}{".exe" if not LINUX else ''}""",
                    "-Xverify:none",
                    f"-Xmx{settings.get('ram', 'Loader')}M",
                    f"-Djava.library.path={native_path}",
                    f"-cp", # Separate -cp flag
                    f"{classpath}{path_sep}.{bc}{self.jar}", # Classpath 
                    self.main_class, # Main class as a separate argument
                    f"--username {settings.get('nickname')}",
                    "--gameDir ./",
                    f"--assetsDir",
                    asset_path,
                    f"--assetIndex",
                    self.version,
                    "--uuid N/A",
                    "--accessToken",
                    "0",
                    "--userType legacy",
                    f"--version",
                    self.version
                ]
                
                if LINUX:
                    java_command = [arg.replace('\\', '/') for arg in java_command]

                command = ' '.join(java_command)
                self.debug(command)

                process = Popen(java_command, stdout=PIPE, stderr=STDOUT)
                buffer = []

                for line in process.stdout:
                    _ = line.decode('utf-8', errors='ignore')
                    progress.print(_, end='', markup=False, highlight=False)
                    buffer.append(_)

                logchecker.check_logs(buffer)
 
                self.info('Exited from minecraft')

        # Return default title
        selector.reset_title()
