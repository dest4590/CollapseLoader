
from ..modules.Module import Module


class LogChecker(Module):
    """Used to check game logs to detect crashes"""

    def __init__(self) -> None:
        """Initialize LogChecker and log the initialization"""
        super().__init__()
        self.debug('Initialized LogChecker')

    def reason(self, msg: str) -> str:
        """Construct a reason message for the crash"""
        return f'Game crashed because {msg}'

    def check_logs(self, payload: str) -> None:
        """Check logs for crash messages and log appropriate errors and info"""
        self.debug('Checking log')
        logs = ''.join(payload)

        if 'Game crashed!' in logs:
            self.error('Game crashed!')

            if 'java.lang.OutOfMemoryError: Java heap space' in logs:
                self.info(self.reason('it has too little memory'))

            if 'java.lang.NoClassDefFoundError' in logs:
                self.info(self.reason('some libraries are missing'))
        else:
            self.debug('No crashes detected, all good!')

logchecker = LogChecker()
