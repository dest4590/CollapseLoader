from rich import print

from .utils.Logo import logo
from .utils.Selector import selector

# Using rich library for displaying bold and color texts
print('[bold white]' + logo.full)
print('[bold green]' + logo.tagline)

while True:
    selector.show()
    choosed = selector.select()

    # TODO