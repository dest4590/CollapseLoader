import json
import os
from datetime import datetime

from ...config import ROOT_DIR
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
            payload = {"clients": clients, "_meta": {"creation_time": now}}

            with open(self.path, "w", encoding="utf-8") as f:
                json.dump(payload, f)

            self.debug(lang.t("cache.cache-saved").format(now))

    def get(self) -> dict:
        """Returns cache as dict"""
        with open(self.path, "r", encoding="utf-8") as f:
            return json.load(f)


cache = Cache()
