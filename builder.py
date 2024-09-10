import os
import shutil
from glob import glob


class Builder:
    """Class to build to .exe file"""

    def __init__(self, onefile: bool = True, clean: bool = True, name: str = 'CollapseLoader', icon: str = 'logo.ico'):
        self.onefile = onefile
        self.clean = clean
        self.name = name
        self.icon = icon

    def build(self, removeBuild: bool = True):
        """Starts the build"""
        # Remove old .exe builds
        for file in glob('*.exe'):
            os.remove(file)
        
        os.system(f'''pyinstaller --onefile --clean --console --name "{self.name}" --icon "collapse\\assets\\{self.icon}" run.py''')

        for file in glob('dist/*.exe'):
            shutil.move(file, './')

        if removeBuild:
            self.clear_all()

    def clear_all(self):
        """Deletes all unnecessary files"""
        if os.path.exists('build/'):
            shutil.rmtree('build/')

        if os.path.exists('dist/'):
            shutil.rmtree('dist/')

        # Remove spec file
        for spec in glob('*.spec'):
            os.remove(spec)


Builder().build()
