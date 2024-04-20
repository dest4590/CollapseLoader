from .Cheat import Cheat
from .Data import data
from .API import api

cheats = []

for cheat in api.clients():
    cheats.append(Cheat(cheat['name'], data.get_url(cheat['filename']), cheat['main_class'], cheat['version'], cheat['internal']))