from ..utils.Logger import logger

class Module:
    def __init__(self):
        self.module_name = self.__class__.__name__

    def debug(self):
        logger.debug('[]', okay='GOOD!')
