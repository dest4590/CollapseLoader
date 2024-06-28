import os

from rich.markup import escape

from .API import api
from .Cache import cache
from .Cheat import Cheat
from .Data import data
from .Logger import logger


class CheatManager:
    """Class to manage and load cheats from the API"""

    def __init__(self):
        self.cheats = self._load_cheats()

    def _load_cheats(self):
        """Load cheats from the API and return a list of Cheat instances"""
        clients = api.get('clients')
        cheats = []

        if clients is not None:
            cache.save(clients.json())

            for cheat in clients.json():
                if cheat["show_in_loader"]:
                    cheats.append(
                        Cheat(
                            name=escape(cheat["name"]) + (" [red bold][-][/]" if not cheat["working"] else ""),
                            link=data.get_url(cheat["filename"]),
                            main_class=cheat["main_class"],
                            version=cheat["version"][:-2],
                            category=cheat["category"],
                            internal=cheat["internal"],
                        )
                    )

        else:
            if not os.path.exists(cache.path):
                logger.error('No clients cache found')

            else:
                c = cache.get()
                creation_time = c['_meta']['creation_time']
                logger.info(f"Using latest clients cache ({creation_time})")

                for cheat in c['clients']:
                    if cheat["show_in_loader"]:
                        cheats.append(
                            Cheat(
                                name=escape(cheat["name"]) + (" [red bold][-][/]" if not cheat["working"] else ""),
                                link=data.get_url(cheat["filename"]),
                                main_class=cheat["main_class"],
                                version=cheat["version"][:-2],
                                category=cheat["category"],
                                internal=cheat["internal"],
                            )
                        )

        return cheats

cheat_manager = CheatManager()
