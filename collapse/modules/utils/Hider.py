import ctypes

from ..utils.Module import Module


class Hider(Module):
    """Class for hiding loader"""
    
    def __init__(self) -> None:
        super().__init__()
    
    def hide_window(self):
        ctypes.windll.kernel32.FreeConsole()

    def show_window(self):
        ctypes.windll.kernel32.AllocConsole()

hider = Hider()