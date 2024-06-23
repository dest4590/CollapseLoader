from .Logger import logger

class LogChecker:
    """Used to check game logs to detect crashes"""

    def __init__(self):
        """Initialize LogChecker and log the initialization"""
        logger.debug('Initialized LogChecker')

    def reason(self, msg: str) -> str:
        """Construct a reason message for the crash"""
        return f'Game crashed because {msg}'

    def checklogs(self, payload: str) -> None:
        """Check logs for crash messages and log appropriate errors and info"""
        logger.debug('Checking log')
        logs = ''.join(payload)

        if 'Game crashed!' in logs:
            logger.error('Game crashed!')

            if 'java.lang.OutOfMemoryError: Java heap space' in logs:
                logger.info(self.reason('it has too little memory'))
        else:
            logger.debug('No crashes detected, all good!')

logchecker = LogChecker()
