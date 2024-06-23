from rich.markup import escape

from .API import api
from .Cheat import Cheat
from .Data import data


class CheatManager:
    """A class to manage and load cheats from the API"""

    def __init__(self):
        self.cheats = self.load_cheats()

    @staticmethod
    def load_cheats():
        """Load cheats from the API and return a list of Cheat instances"""
        cheats = []
        for cheat in api.get('clients').json():
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