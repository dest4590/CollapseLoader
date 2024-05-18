from .Logger import logger


class LogChecker:
    def __init__(self):
        logger.debug('Initialized LogChecker')

    def reason(self, msg: str) -> str:
        return f'Game crashed because {msg}'

    def checklogs(self, payload: str) -> str:
        logger.debug('Checking log')

        logs = ''.join(payload)

        if 'Game crashed!' in logs:
            logger.error('Game crashed!')
            
            if 'java.lang.OutOfMemoryError: Java heap space' in logs:
                logger.info(self.reason('has too little memory'))

logchecker = LogChecker()