from .Logger import logger

class LogChecker:
    def __init__(self):
        logger.debug('Initialized LogChecker')

    def check(self, payload: str) -> str:
        logger.debug('Checking log')

        