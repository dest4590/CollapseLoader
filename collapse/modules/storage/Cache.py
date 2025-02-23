import json
import os
from datetime import datetime

from ...config import CODENAME, ROOT_DIR, VERSION
from ..utils.Fixes import console
from ..utils.Language import lang
from ..utils.Module import Module
from .Settings import settings


class Cache(Module):
    """Class for clients caching"""

    def __init__(self, file: str = "cache.json") -> None:
        super().__init__()
        self.file = file
        self.path = os.path.join(ROOT_DIR, file)

    def save(self, clients: list) -> None:
        """Saves cache into file"""

        if settings.use_option("disable_caching"):
            now = datetime.now().strftime("%d/%m/%Y %H:%M:%S")
            payload = {
                "clients": clients,
                "_meta": {"creation_time": now, "version": f"{VERSION} ({CODENAME})"},
            }

            with open(self.path, "w", encoding="utf-8") as f:
                json.dump(payload, f)

            self.info(lang.t("cache.cache-saved").format(now))

    def clear(self) -> None:
        """Clears cache"""
        if os.path.exists(self.path):
            os.remove(self.path)

    def display_info(self) -> None:
        """Prints cache info"""
        if os.path.exists(self.path):
            with open(self.path, "r", encoding="utf-8") as f:
                data = json.load(f)
                creation_time = data["_meta"]["creation_time"]
                version = data["_meta"]["version"]
                clients = len(data["clients"])
                size = round(os.path.getsize(self.path) / 1024, 2)

                console.print(
                    lang.t("cache.cache-info").format(
                        size, clients, creation_time, version
                    )
                )
        else:
            self.warn(lang.t("cache.cache-not-found"))

    def get(self) -> dict:
        """Returns cache as dict"""
        with open(self.path, "r", encoding="utf-8") as f:
            return json.load(f)


cache = Cache()
