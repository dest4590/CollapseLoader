import os
import shutil
from glob import glob


class Builder:
    def __init__(self, onefile: bool = True, clean: bool = True, name: str = 'CollapseLoader', icon: str = 'logo.ico'):
        self.onefile = onefile
        self.clean = clean
        self.name = name
        self.icon = icon
    
    def build(self, removeBuild: bool = True, skipUPX: bool = False):
        os.system(f'''pyinstaller --onefile --clean --console {'--upx-dir ".\\upx"' if not skipUPX else ''} --name "{self.name}" --icon "collapse\\assets\\{self.icon}" run.py''')

        for file in glob('dist/*.exe'):
            shutil.move(file, './')

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


Builder().build(skipUPX=True)