import traceback
import os

dev = False

if not dev:
    try:
        import collapse
    except Exception as e:
        print('An error occurred:')

        print('==========')
        traceback.print_exc()
        print('==========')

        os.system('pause')

if dev:
    import collapse