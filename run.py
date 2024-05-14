import os
import traceback

dev = True

if not dev:
    try:
        import collapse.main
    except Exception as e:
        print('An error occurred:')

        print('==========')
        traceback.print_exc()
        print('==========')

        os.system('pause')

elif dev:
    import collapse.main
