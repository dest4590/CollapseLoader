import os
import time
from contextlib import contextmanager
from subprocess import PIPE, STDOUT, Popen

from .Data import data
from .LogChecker import logchecker
from .Logger import logger
from .RPC import rpc
from .Settings import settings


@contextmanager
def chdir(directory):
    """Context manager for changing the current working directory"""
    old_dir = os.getcwd()
    try:
        os.chdir(directory)
        yield
    finally:
        os.chdir(old_dir)


class Cheat:
    def __init__(
        self,
        name: str,
        link: str,
        main_class: str = "net.minecraft.client.main.Main",
        version: str = "1.12.2",
        category: str = "HVH",
        internal: bool = False,
    ) -> None:
        self.name = name
        self.link = link
        self.category = category

        self.filename = os.path.basename(self.link)
        self.path = os.path.join(data.root_dir, self.filename)
        self.path_dir = os.path.join(data.root_dir, os.path.splitext(self.filename)[0])
        self.jar = os.path.splitext(self.filename)[0] + ".jar"

        self.main_class = main_class
        self.version = version
        self.internal = internal
        self.silent = False

    def __str__(self) -> str:
        return self.name

    def download(self) -> bool:
        """Download cheat files"""
        if os.path.isfile(os.path.join(self.path_dir, self.jar)):
            logger.debug(f"Client {self.name} already downloaded")
            return True

        logger.info("Downloading client")
        data.download(self.filename)
        return True

    def run(self):
        """Run the client"""
        rpc.details = f"Running {self.name}"
        rpc.start_time = time.time()

        # Downloading requirements
        data.download("jre-21.0.2.zip")

        if self.version.startswith("1.12"):
            logger.debug("Downloading 1.12.2 libraries & natives")
            data.download("libraries-1.12.zip")
            data.download("natives-1.12.zip")
        else:
            logger.debug("Downloading 1.12.2+ libraries & natives")
            data.download("libraries.zip")
            data.download("natives.zip")

        data.download("assets.zip")

        logger.info(f"Running client {self.name}")
        with chdir(self.path_dir):
            bc = "\\" if os.name == "nt" else "/"
            path_sep = ";" if os.name == "nt" else ":"

            classpath = (
                f".{bc}libraries-1.12{bc}*"
                if self.version.startswith("1.12")
                else f".{bc}libraries{bc}*"
            )
            native_path = (
                f".{bc}natives-1.12;"
                if self.version.startswith("1.12")
                else f".{bc}natives;"
            )
            asset_path = f".{bc}assets"

            java_command = [
                f"..{bc}jre-21.0.2{bc}bin{bc}java{'w' if self.silent else ''}.exe",
                "-Xverify:none",
                f"-Xmx{settings.get('ram')}M",
                f"-Djava.library.path={native_path}",
                f"-cp {classpath}{path_sep}.{bc}{self.jar} {self.main_class}",
                f"--username {settings.get('nickname', 'Options')}",
                "--gameDir .\\",
                f"--assetsDir {asset_path}",
                f"--assetIndex {self.version}",
                "--uuid N/A",
                "--accessToken 0",
                "--userType legacy",
                f"--version {self.version}",
            ]

            command = " ".join(java_command)
            logger.debug(command)

            process = Popen(command, stdout=PIPE, stderr=STDOUT)
            buffer = []

            for line in process.stdout:
                output = line.decode("utf-8", errors="ignore")
                print(output, end="")
                buffer.append(output)

            logchecker.checklogs(buffer)
            logger.info("Exited from Minecraft")

            rpc.start_time = time.time()
            rpc.details = "Choosing a client"
