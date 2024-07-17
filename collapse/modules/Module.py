from ..utils.Logger import logger

class Module:
    """The module is made to simplify the use of logging, etc."""

    def __init__(self):
        self.__module_name = self.__class__.__name__

    def debug(self, msg: object):
        """Make a debug log"""
        logger.debug(msg=msg, prefix=f' # {self.__module_name}')

    def info(self, msg: object):
        """Make a info log"""
        logger.info(msg=msg, prefix=f' * {self.__module_name}')

    def warn(self, msg: object):
        """Make a warn log"""
        logger.warn(msg=msg, prefix=f' ! {self.__module_name}')

    def error(self, msg: object):
        """Make a error log"""
        logger.error(msg=msg, prefix=f' & {self.__module_name}')

    def critical(self, msg: object):
        """Make a critical log"""
        logger.critical(msg=msg, prefix=f' !!! {self.__module_name}')