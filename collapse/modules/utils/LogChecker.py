from .Module import Module


class LogChecker(Module):
    """Used to check game logs to detect crashes"""

    def __init__(self) -> None:
        """Initialize LogChecker and log the initialization"""
        super().__init__()

    def reason(self, msg: str) -> str:
        """Construct a reason message for the crash"""
        return f'Game crashed because {msg}'

    def check_logs(self, payload: str) -> None:
        """Check logs for crash messages and log appropriate errors and info"""
        self.debug('Checking log')
        logs = ''.join(payload)

        if 'Game crashed!' in logs:
            self.error('Game crashed!')

            # Memory Errors
            if 'java.lang.OutOfMemoryError: Java heap space' in logs:
                self.info(self.reason('it has too little heap memory'))
            elif 'java.lang.OutOfMemoryError: Metaspace' in logs:
                self.info(self.reason('it ran out of memory for loading classes'))
            elif 'java.lang.OutOfMemoryError: GC overhead limit exceeded' in logs:
                self.info(self.reason('the garbage collector is spending too much time collecting garbage'))

            # Class and Library Errors
            elif 'java.lang.NoClassDefFoundError' in logs:
                self.info(self.reason('some required classes or libraries are missing'))
            elif 'java.lang.UnsupportedClassVersionError' in logs:
                self.info(self.reason('it was compiled with a different Java version'))

            # Modding Errors
            elif 'LoaderExceptionModCrash' in logs:
                self.info(self.reason('a mod caused a crash'))
            elif ' ticking entity' in logs:
                self.info(self.reason("an entity is causing issues (often a mod-related problem)"))
            
            # World Corruption
            elif 'Failed to load level' in logs:
                self.info(self.reason('your world data might be corrupted'))
            
            # Graphics and Rendering
            elif 'Pixel format not accelerated' in logs:
                self.info(self.reason('there might be an issue with your graphics card or drivers'))
            elif 'OpenGL error' in logs:
                self.info(self.reason("an OpenGL error occurred, often graphics driver related"))

            # Catch-all for unknown reasons
            else:
                self.warn('Game crashed for an unknown reason. '
                              'Please provide the full log file for further analysis.')
            
            return True
        else:
            self.debug('No crashes detected, all good!')
            return False

logchecker = LogChecker()