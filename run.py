import os
import traceback

dev = True

if not dev:
    try:
        from collapse.main import main
        main()
    except Exception as e:
        print('An error occurred:')

        print('==========')
        traceback.print_exc()
        print('==========')

        os.system('pause')

elif dev:
    from collapse.main import main
    main()