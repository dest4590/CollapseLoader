from collapse.modules.network.API import api
from collapse.modules.utils.Language import lang
from collapse.modules.utils.Module import Module


class HeaderText(Module):
    def __init__(self):
        super().__init__()
        
        self.text = None

    def get(self):
        if self.text is None:
            fetched_text = api.get(f'header/?lang={lang.current}', prefix=False)
        
        if fetched_text is not None:
            self.text = fetched_text.text
        
        return self.text
    
header = HeaderText()