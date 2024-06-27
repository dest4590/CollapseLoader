import logging

import colorlog

from ..static import DEBUG_LOGS


class CollapseLogger(logging.Logger):
    """Logging with custom levels and colored output"""

    API_LEVEL = 11
    logging.addLevelName(API_LEVEL, 'API')

    def __init__(self, name: str, level: int = logging.DEBUG) -> None:
        super().__init__(name, level)
        self._setup_logger()

    def _setup_logger(self) -> None:
        """Set up the logger with colored output."""
        log_colors = {
            'DEBUG': 'cyan',
            'INFO': 'green',
            'WARNING': 'yellow',
            'ERROR': 'red',
            'CRITICAL': 'red,bg_white',
            'API': 'light_cyan'
        }

        secondary_log_colors = {
            'message': {
                'ERROR': 'red',
                'CRITICAL': 'red',
                'WARNING': 'yellow',
                'INFO': 'blue',
                'API': 'light_cyan',
                'DEBUG': 'green',
            }
        }

        formatter = colorlog.ColoredFormatter(
            "[%(log_color)s%(levelname)s%(reset)s] %(message_log_color)s%(message)s",
            datefmt=None,
            reset=True,
            log_colors=log_colors,
            secondary_log_colors=secondary_log_colors,
            style='%'
        )

        handler = logging.StreamHandler()
        handler.setFormatter(formatter)

        self.addHandler(handler)

    def api(self, msg: object) -> None:
        """Log a message with the custom API level."""
        self.log(self.API_LEVEL, msg)

# Create a logger instance
logger = CollapseLogger('CollapseLogger', logging.DEBUG if DEBUG_LOGS else logging.INFO)
