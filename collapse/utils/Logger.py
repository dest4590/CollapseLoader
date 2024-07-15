import logging

import colorlog


class CollapseLogger(logging.Logger):
    """Logging with custom levels and colored output"""

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

        class OptionalPrefixFormatter(colorlog.ColoredFormatter):
            """Class that adds a prefix attribute to log records if it doesn't already exist."""
            def format(self, record):
                record.prefix = record.__dict__.get('prefix', '')
                return super().format(record)

        formatter = OptionalPrefixFormatter(
            "[%(log_color)s%(levelname)s%(prefix)s%(reset)s] %(message_log_color)s%(message)s",
            datefmt=None,
            reset=True,
            log_colors=log_colors,
            secondary_log_colors=secondary_log_colors,
            style='%'
        )

        handler = logging.StreamHandler()
        handler.setFormatter(formatter)

        self.addHandler(handler)

    def _log(self, level, msg, args, **kwargs):
        """Rewrite function of logging system"""
        prefix = kwargs.pop('prefix', '')
        kwargs['extra'] = {'prefix': prefix}
        super()._log(level, msg, args, **kwargs)

# Create a logger instance
logger = CollapseLogger('CollapseLogger', level=logging.INFO)
