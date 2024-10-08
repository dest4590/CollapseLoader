from collapse.modules.network.API import api
from collapse.modules.utils.Language import lang
from collapse.modules.utils.Module import Module


class HeaderText(Module):
    def __init__(self):
        super().__init__()
        
        self.text = None
        
    def get(self):
        if self.text is None:
            self.text = api.get(f'header/?lang={lang.current}', prefix=False).text
        
        return self.text
    
header = HeaderText()