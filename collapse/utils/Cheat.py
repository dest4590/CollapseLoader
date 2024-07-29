import os
from contextlib import chdir
from datetime import datetime
from subprocess import PIPE, STDOUT, Popen
from threading import Thread
from time import sleep

from rich.progress import BarColumn, Progress, SpinnerColumn, TextColumn

from .Data import console, data
from .LogChecker import logchecker
from .Logger import logger
from .Settings import settings


def update_time(task_id, progress, start_time) -> None:
    """Updates the time for the progress bar by ticking one second"""
    while True:
        elapsed_time = datetime.now() - start_time
        progress.update(
            task_id,
            time=str(elapsed_time).split('.', maxsplit=1)[0])

        sleep(1)


class Cheat:
    """Cheat class for running clients"""

    def __init__(self, name: str, link: str,
                 main_class: str = 'net.minecraft.client.main.Main',
                 version: str = '1.12.2',
                 category: str = 'HVH', internal: bool = False) -> None:

        self.name = name
        self.link = link
        self.category = category

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
            logger.debug(f'Client {self.name} already downloaded')
            return

        logger.info('Downloading client')

        data.download(self.filename)

    def run(self) -> None:
        """Run client"""

        self.download()

        from .CLI import selector

        selector.set_title(selector.titles_states['run'].format(client=self.name))

        # Downloading requirements
        data.download('jre-21.0.2.zip')

        if self.version.startswith('1.12'):
            logger.debug('Downloading 1.12.2 libraries & natives')
            data.download('libraries-1.12.zip')
            data.download('natives-1.12.zip')

        else:
            logger.debug('Downloading 1.12.2+ libraries & natives')
            data.download('libraries.zip')
            data.download('natives.zip')

        data.download('assets.zip')

        logger.info(f'Running client {self.name}')

        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
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

            with chdir('.\\' + self.path_dir):
                # Using backslash var, because f-strings not supporting it in expressions
                # pylint: disable=line-too-long
                bc = '\\'

                path_sep = ';' if os.name == 'nt' else ':'

                if self.internal and os.path.isdir('libraries'):
                    classpath = f'.{bc}libraries-1.12{bc}*' if self.version.startswith('1.12') else f'.{bc}libraries{bc}*'
                else:
                    classpath = f'..{bc}libraries-1.12{bc}*' if self.version.startswith('1.12') else f'..{bc}libraries{bc}*'

                if self.internal and os.path.isdir('natives'):
                    native_path = f'.{bc}natives-1.12;' if self.version.startswith('1.12') else f'.{bc}natives;'
                else:
                    native_path = f'..{bc}natives-1.12;' if self.version.startswith('1.12') else f'..{bc}natives;'

                asset_path = f'.{bc}assets' if self.internal and os.path.isdir('assets') else f'..{bc}assets'

                java_command = [
                    f"..\\jre-21.0.2\\bin\\java{'w' if self.silent else ''}.exe",
                    "-Xverify:none",
                    f"-Xmx{settings.get('ram', 'Loader')}M",
                    f"-Djava.library.path={native_path}",
                    f"-cp {classpath}{path_sep}.{bc}{self.jar} {self.main_class}",
                    f"--username {settings.get('nickname')}",
                    "--gameDir .\\",
                    f"--assetsDir {asset_path}",
                    f"--assetIndex {self.version}",
                    "--uuid N/A",
                    "--accessToken 0",
                    "--userType legacy",
                    f"--version {self.version}"
                ]

                command = ' '.join(java_command)
                logger.debug(command)

                process = Popen(command, stdout=PIPE, stderr=STDOUT)
                buffer = []

                for line in process.stdout:
                    _ = line.decode('utf-8', errors='ignore')
                    progress.print(_, end='', markup=False, highlight=False)
                    buffer.append(_)

                logchecker.checklogs(buffer)

                logger.info('Exited from minecraft')

        # Return default title
        selector.reset_title()
