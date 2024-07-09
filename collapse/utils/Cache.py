import json
from datetime import datetime

from .Data import data
from .Settings import settings

class Cache:
    """Class for clients caching"""
    def __init__(self, file: str = 'cache.json') -> None:
        self.file = file
        self.path = data.get_local(file)

    def save(self, clients: list) -> None:
        """Saves cache into file"""

        if settings.use_option('disable_caching'):
            # Add timestamp to cache
            payload = {'clients': clients, '_meta': {'creation_time': datetime.now().strftime('%d/%m/%Y %H:%M:%S')}}

            with open(self.path, 'w', encoding='utf-8') as f:
                json.dump(payload, f)

    def get(self) -> dict:
        """Returns cache as dict"""
        with open(self.path, 'r', encoding='utf-8') as f:
            return json.load(f)


cache = Cache()
