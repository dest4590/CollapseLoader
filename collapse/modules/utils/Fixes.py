import rich
from rich.console import Console

console = Console()

# Fix rich render, https://github.com/Textualize/rich/pull/3038#issuecomment-1786654627
if console.legacy_windows:
    try:
        import ctypes

        from rich import console as conlib
        from rich._win32_console import (ENABLE_VIRTUAL_TERMINAL_PROCESSING,
                                         GetConsoleMode, GetStdHandle)

        windll = ctypes.LibraryLoader(ctypes.WinDLL)

        handle = GetStdHandle()
        mode = GetConsoleMode(handle)

        mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING

        SetConsoleMode = windll.kernel32.SetConsoleMode
        SetConsoleMode.argtypes = [
            ctypes.wintypes.HANDLE,
            ctypes.wintypes.DWORD,
        ]
        SetConsoleMode.restype = ctypes.wintypes.BOOL
        SetConsoleMode(handle, mode)

        conlib._windows_console_features = None
        console = Console()
    except rich._win32_console.LegacyWindowsError:
        pass
