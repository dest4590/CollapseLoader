import os
import sys
import traceback
from datetime import datetime

from collapse.config import ROOT_DIR

dev = True

if not dev:
    try:
        from collapse.main import main
        main()

    except Exception:
        exc_type, exc_value, exc_traceback = sys.exc_info()
        exc_lines = traceback.format_exception(exc_type, exc_value, exc_traceback)
        max_length = max(len(line) for line in exc_lines)
        
        print('=' * max_length)
        traceback.print_exception(exc_type, exc_value, exc_traceback)
        print('=' * max_length)

        if not os.path.exists(f'{ROOT_DIR}/crash_logs'):
            os.mkdir(f'{ROOT_DIR}/crash_logs')

        with open(f'{ROOT_DIR}/crash_logs/crash_{datetime.now().strftime("%d-%m-%Y_%H-%M-%S")}.log', 'w') as f:
            f.write(''.join(exc_lines))

        os.system('pause')

elif dev:
    from collapse.main import main
    main()