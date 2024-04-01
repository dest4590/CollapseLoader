from .utils.Logo import Logo
from rich import print

logo = Logo()

# Using rich library for displaying bold and color texts
print('[bold white]' + logo.full)
print('[bold green]' + logo.tagline)