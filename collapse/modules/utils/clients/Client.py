import os
import shutil
from contextlib import chdir
from datetime import datetime, timedelta
from subprocess import PIPE, STDOUT, Popen
from threading import Thread
from time import sleep

from rich.progress import BarColumn, Progress, TextColumn

from ...network.Analytics import analytics
from ...storage.Data import console, data
from ...storage.Settings import settings
from ..Fixes import console
from ..Language import lang
from ..LogChecker import logchecker
from ..Module import Module
from ..RPC import rpc


def update_time(task_id, progress: Progress, start_time: datetime) -> None:
    """Update the time field in the progress bar."""
    while not progress.tasks[task_id].finished:
        elapsed_time = datetime.now() - start_time
        formatted_time = str(timedelta(seconds=int(elapsed_time.total_seconds())))
        progress.update(task_id, time=formatted_time)
        sleep(1)


class Client(Module):
    """Client class for running clients"""

    def __init__(
        self,
        name: str,
        link: str,
        main_class: str = "net.minecraft.client.main.Main",
        version: str = "1.12.2",
        internal: bool = False,
        working: bool = True,
        id: int = 1,
        fabric: bool = False,
    ) -> None:
        super().__init__(False)

        self.name = name
        self.link = link
        self.working = working
        self.id = id
        self.fabric = fabric
        self.configs = []

        self.filename = os.path.basename(self.link)
        self.path = data.root_dir + self.filename
        self.path_dir = data.root_dir + os.path.splitext(self.filename)[0] + "/"
        self.jar = os.path.splitext(self.filename)[0] + ".jar"

        self.main_class = main_class

        self.cut_version = True
        self.version = version[:-2] if self.cut_version and not fabric else version

        self.internal = internal
        self.silent = False

    def __str__(self) -> str:
        is_downloaded = data.boolean_states[data.is_downloaded(self.filename)]
        version = (
            f" <{self.version}>"
            if not settings.use_option("show_client_version")
            else ""
        )
        return f"{self.name}{version}{is_downloaded if not settings.use_option('show_installed') else ''}"

    def to_dict(self) -> dict:
        attributes = vars(self)
        attributes["configs"] = None
        attributes["mod_manager"] = None
        return attributes

    def load_config(self, config) -> None:
        """Load configurations for the client"""

        if not os.path.exists(self.path_dir + config.config_path):
            os.makedirs(self.path_dir + config.config_path)

        data.download(
            config.file,
            os.path.join(self.path_dir, config.config_path, config.filename),
            True,
        )

    def download(self) -> True:
        """Downloading client files"""

        if os.path.isfile(self.path_dir + self.jar):
            self.debug(lang.t("clients.already-downloaded").format(self.name))
            return

        else:
            self.info(lang.t("clients.downloading").format(self.name))

        data.download(self.filename)

        from ...render.CLI import selector

        selector.refresh_text()

    def reset(self) -> None:
        """Reset the client"""
        if os.path.isdir(self.path_dir):
            shutil.rmtree(self.path_dir)

        self.download()

    def delete(self) -> None:
        """Delete the client"""
        if os.path.isdir(self.path_dir):
            shutil.rmtree(self.path_dir, ignore_errors=True)

    def open_folder(self) -> None:
        """Open the client folder"""
        if os.path.isdir(self.path_dir):
            absolute_path = os.path.abspath(self.path_dir)
            Popen(f"explorer /open,{absolute_path}")

    def run(self) -> None:
        """Run client"""

        from ...render.CLI import selector

        selector.set_title(selector.titles_states["run"].format(client=self.name))

        rpc.details = lang.t("rpc.playing").format(self.name)

        # Downloading requirements
        data.download("jre-21.0.2.zip")

        if self.version.startswith("1.12"):
            self.info(lang.t("clients.downloading-libraries-natives-1-12"))
            data.download("libraries-1.12.zip")
            data.download("natives-1.12.zip")

        else:
            self.info(lang.t("clients.downloading-libraries-natives-1-12-2"))
            data.download("libraries.zip")
            data.download("natives.zip")

        data.download("assets.zip")

        self.download()

        self.info(lang.t("clients.running").format(self.name))

        analytics.client_run(self.id)

        with Progress(
            TextColumn("[progress.description]{task.description}"),
            BarColumn(pulse_style="gray"),
            TextColumn("{task.fields[session]} {task.fields[time]}"),
            transient=True,
            console=console,
        ) as progress:
            start_time = datetime.now()
            task_id = progress.add_task(
                lang.t("clients.progress.task").format(
                    self.name, settings.get("nickname")
                ),
                session=lang.t("clients.progress.session"),
                time="00:00:00",
                total=None,
            )

            Thread(
                target=update_time, args=(task_id, progress, start_time), daemon=True
            ).start()

            with chdir(".\\" + self.path_dir):
                # Using backslash var, because f-strings not supporting it in expressions
                bc = "\\"

                path_sep = ";" if os.name == "nt" else ":"

                libraries_dir = (
                    "libraries-1.12" if self.version.startswith("1.12") else "libraries"
                )
                natives_dir = (
                    "natives-1.12" if self.version.startswith("1.12") else "natives"
                )
                assets_dir = "assets"

                if self.internal and os.path.isdir(libraries_dir):
                    classpath = f".{bc}{libraries_dir}{bc}*"
                else:
                    classpath = f"..{bc}{libraries_dir}{bc}*"

                if self.internal and os.path.isdir(natives_dir):
                    native_path = f".{bc}{natives_dir};"
                else:
                    native_path = f"..{bc}{natives_dir};"

                asset_path = (
                    f".{bc}{assets_dir}"
                    if self.internal and os.path.isdir(assets_dir)
                    else f"..{bc}{assets_dir}"
                )

                java_command = [
                    f"..\\jre-21.0.2\\bin\\java{'w' if self.silent else ''}.exe",
                    "-Xverify:none",
                    f"-Xmx{settings.get('ram')}M",
                    f"-Djava.library.path={native_path}",
                    f"-cp {classpath}{path_sep}.{bc}{self.jar} {self.main_class}",
                    f"--username {settings.get('nickname')}",
                    "--gameDir .\\",
                    f"--assetsDir {asset_path}",
                    f"--assetIndex {self.version}",
                    "--uuid 00000000-0000-0000-0000-000000000000",
                    "--accessToken 0",
                    "--userType legacy",
                    f"--version {self.version}",
                ]

                command = " ".join(java_command)

                selector.hide_console()

                process = Popen(command, stdout=PIPE, stderr=STDOUT)
                buffer = []

                for line in process.stdout:
                    _ = line.decode("utf-8", errors="ignore")
                    progress.print(_, end="", markup=False, highlight=False)
                    buffer.append(_)

                logchecker.check_logs(buffer, self)

                self.info(lang.t("clients.finished"))

        # Return default title
        selector.reset_title()

        selector.show_console()

        rpc.details = rpc.default_details
        rpc.update()
