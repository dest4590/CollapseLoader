import logging

import colorlog

from ..static import DEBUG_LOGS

# Custom log level
API = 11
logging.addLevelName(API, 'API')

def setup_logger(name, level=logging.DEBUG):
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

    logger = logging.getLogger(name)
    logger.setLevel(level)
    logger.addHandler(handler)

    return logger

logger = setup_logger('CollapseLogger', logging.DEBUG if DEBUG_LOGS else logging.INFO)
 
