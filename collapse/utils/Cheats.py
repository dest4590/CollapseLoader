from .Cheat import Cheat
from .Data import data

celestialrecode = Cheat('CelestialRecode', data.get_url('CelestialRecode.zip'), version='1.16.5', internal=True, use_as_jar=True, use_libraries=False)
lifeclient = Cheat('LifeClient', data.get_url('LifeClient.zip'), internal=True)
rockstar = Cheat('Rockstar', data.get_url('Rockstar.zip'), internal=True)
moonproject = Cheat('MoonProject', data.get_url('MoonProject.zip'))
wild = Cheat('Wild', data.get_url('WildDLC.zip'))
speedclient = Cheat('SpeedClient', data.get_url('SpeedClient.zip'))

# Don't forget to add cheats to this array
cheats = [
    celestialrecode, 
    lifeclient, 
    rockstar,
    moonproject,
    wild,
    speedclient
]