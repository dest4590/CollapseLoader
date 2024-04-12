import os

class ModManager:
    def __init__(self, root_folder: str):
        self.root_folder = root_folder

    def get_mod_list(self):
        return os.listdir(self.root_folder)