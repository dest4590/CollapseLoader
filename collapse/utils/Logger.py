import logging
import colorlog

API = 11

logging.addLevelName(API, 'API')

def setup_logger(name, level):

    formatter = colorlog.ColoredFormatter(
        "[%(log_color)s%(levelname)s%(reset)s] %(message_log_color)s%(message)s",
        datefmt=None,
        reset=True,
        log_colors={'DEBUG': 'cyan', 'INFO': 'green', 'WARNING': 'yellow', 'ERROR': 'red', 'CRITICAL': 'red,bg_white', 'API': 'light_yellow'},
        secondary_log_colors={
            'message': {
                'ERROR':    'red',
                'CRITICAL': 'red',
                'INFO':     'blue',
                'API':      'light_yellow',
                'DEBUG':    'green',
            }
        },
        style='%',
    )

    handler = logging.StreamHandler()
    handler.setFormatter(formatter)

    logger = logging.getLogger(name)

    logger.setLevel(level)
 
    logger.addHandler(handler)

    return logger

logger = setup_logger('CollapseLogger', logging.DEBUG)
