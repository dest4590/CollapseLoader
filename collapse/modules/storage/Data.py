import os
import random
import zipfile

import requests
from rich.progress import (BarColumn, DownloadColumn, Progress, SpinnerColumn,
                           TextColumn, TransferSpeedColumn)

from ...static import CODENAME, REPO_URL, VERSION
from ..network.Network import network
from ..network.Servers import servers
from ..utils.Fixes import console
from ..utils.Module import Module


class DataManager(Module):
    """Used to manage loader data"""
    def __init__(self) -> None:
        super().__init__()
        self.root_dir = 'data/'
        self.server = servers.check_servers()

        if not self.server:
            self.critical('No server was found for downloading files (this is a critical function in the loader)')

        self.repo = REPO_URL
        self.version = VERSION
        self.codename = CODENAME

        os.makedirs(self.root_dir, exist_ok=True)

    def get_local(self, path: str) -> str:
        """Get file locally"""
        return os.path.join(self.root_dir, path)

    def get_url(self, path: str) -> str:
        """Gets a link from the web"""
        return self.server + path
    
    def download(self, path: str, destination: str = None, raw: bool = False) -> None:
        """Downloads file using path"""        
        filename = os.path.basename(path)
        path_dir = os.path.join(self.root_dir, os.path.splitext(filename)[0])
        dest = destination if destination else os.path.join(self.root_dir, filename)

        if self._is_downloaded(filename, path, path_dir):
            return

        self.debug(f'Downloading {filename} to {dest}')
        
        self._download_file(path, filename, dest, raw)
        self._extract_file(filename, dest, path_dir, raw)

    def _is_downloaded(self, filename: str, path: str, path_dir: str) -> bool:
        """Checks if the file is already downloaded."""
        jar = os.path.splitext(filename)[0] + '.jar'

        if not filename.endswith('.jar') and os.path.isdir(path_dir) and not path.startswith('http'):
            self.debug(f'{filename} already downloaded, skip')
            return True
        
        if filename.endswith('.jar') and os.path.exists(os.path.join(path_dir, jar)):
            self.debug(f'{filename} file already downloaded, skip')
            return True

        return False

    def _download_file(self, path: str, filename: str, dest: str, raw: bool = False) -> None:
        """Downloads the file from the given path and shows download progress"""
        if not raw:
            os.makedirs(self.root_dir + os.path.splitext(filename)[0], exist_ok=True)
        
        headers = {'Range': f'bytes={os.path.getsize(dest)}-'} if os.path.exists(dest) else {}

        try:
            response = network.get(self.get_url(filename) if not path.startswith('http') else path, headers=headers, stream=True)
            response.raise_for_status()
            total_size = int(response.headers.get('content-length', 0))
        except requests.exceptions.RequestException as e:
            self.error(f"Failed to download {filename}: {e}")
            return

        with Progress(
                TextColumn(f'[blue]{filename}'),
                SpinnerColumn(f'dots{random.randint(2, 9)}'),
                BarColumn(),
                DownloadColumn(),
                TransferSpeedColumn(), console=console, transient=True) as progress:
            task = progress.add_task('', total=total_size)
            
            with open(dest, "ab") as f:
                for chunk in response.iter_content(1024):
                    if chunk:
                        f.write(chunk)
                        progress.update(task, advance=len(chunk))
                        
            progress.stop()

    def _extract_file(self, filename: str, dest: str, path_dir: str, raw: bool) -> None:
        """Extracts the downloaded file based on its type"""
        try:
            if filename.endswith('.zip'):
                with zipfile.ZipFile(dest, 'r') as zip_file:
                    zip_file.extractall(path_dir)
                os.remove(dest)
            elif filename.endswith('.jar'):
                if not raw:
                    os.rename(dest, os.path.join(path_dir, filename))
        except (zipfile.BadZipFile, OSError) as e:
            self.error(f"Error processing {dest}: {e}")
            if os.path.exists(dest):
                os.remove(dest)

data = DataManager()
