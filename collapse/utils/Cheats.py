from .Cheat import Cheat
from .Fabric.FabricCheat import FabricCheat
from .Data import data

celestialrecode = Cheat('CelestialRecode', data.get_url('CelestialRecode.zip'), version='1.16', internal=True, use_as_jar=True, use_libraries=False)
rockstar = Cheat('Rockstar', data.get_url('Rockstar.zip'), internal=True)
moonproject = Cheat('MoonProject', data.get_url('MoonProject.zip'), version='1.16')
wild = Cheat('Wild', data.get_url('WildDLC.zip'), version='1.16')
speedclient = Cheat('SpeedClient', data.get_url('SpeedClient.zip'), version='1.16')
hachclient = Cheat('HachClient', data.get_url('Hachclient.zip'), internal=True)
expensive_old = Cheat('Expensive 2.0', data.get_url('Expensive-2.0.jar'), version='1.16')
expensive_new = Cheat('Expensive 3.1 (multiplayer not working)', data.get_url('Expensive-3.1.zip'), version='1.16', internal=True)
expensive_upgrade = Cheat('Expensive Upgrade', data.get_url('ExpensiveUpgrade.jar'), version='1.16')
exclusive = Cheat('Exclusive', data.get_url('Exclusive.jar'), version='1.16')
fluger_old = Cheat('Fluger (old)', data.get_url('Fluger-old.zip'), internal=True)
fluger_new = Cheat('Fluger (new)', data.get_url('Fluger-new.jar'), version='1.16')
thunderhack = FabricCheat('ThunderHack', data.get_url('thunderhack-1.4.jar'))
liquidbounce = FabricCheat('LiquidBounce Nextgen', data.get_url('liquidbounce.jar'))

# Don't forget to add cheats to this array
cheats = [
    celestialrecode,
    rockstar,
    moonproject,
    wild,
    speedclient,
    hachclient,
    expensive_old,
    expensive_new,
    expensive_upgrade,
    exclusive,
    fluger_old,
    fluger_new,
    thunderhack,
    liquidbounce
]