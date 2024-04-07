import os, sys
import shutil
from glob import glob

class Builder:
    def __init__(self, onefile: bool = True, clean: bool = True, name: str = 'CollapseLoader', icon: str = 'logo.ico', version: str = '1.0'):
        self.onefile = onefile
        self.clean = clean
        self.name = name
        self.icon = icon
        self.version = version
    
    def build(self, removeBuild: bool = True):
        os.system(f'''pyinstaller --onefile --clean --console --upx-dir "{sys.path[0]}\\upx" --name "{self.name}_{self.version}" --icon "{self.icon}" run.py''')

        [shutil.move(file, './') for file in glob('dist/*.exe')]

        if removeBuild:
            self.ClearAll()

    def ClearAll(self):
        if os.path.exists('build/'):
            shutil.rmtree('build/')

        if os.path.exists('dist/'):
            shutil.rmtree('dist/')

        # Remove spec file
        for spec in glob('*.spec'):
            os.remove(spec)


Builder(version='1.2.1').build()