from .Cheat import Cheat
from .Data import data

celestial = Cheat('Celestial', data.get_url('Celestial.zip'))
lifeclient = Cheat('LifeClient', data.get_url('LifeClient.jar'))

# Add cheats here
cheats = [celestial, lifeclient]