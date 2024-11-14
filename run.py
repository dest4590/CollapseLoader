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

#                                                            ,1,
#                               ......                     .ifL1
#                            .;1ttttLLf1,                 ;fffff,
#                           :LCLfttfLLLLL:              ,tffffffi
#                          ,CCCLf11ff1tff;            ,1Lffffffff.
#                          iLttt11ifLLLfft.           :iitffffffLi
#                          ;t1111ii1fffff1               ffffft;i1.
#                          ,1iiiiii1tt1it,              ifffff,
#                           ;1i;;;;i1tff:              .fffff1
#                           ,t1ii;:::;i;               ifffff,
#                          .,;11iii;:                 .fffff1
#                     ..,,.,,.,iftitGi...             ifffff,
#                 .,,,::::,,,,..;LLLGt::::,,,.       .fffff1
#                 ;,,,,,,,,,,,,,.,;tfi,::,,::::,     ;fffff,
#                 ::,,,,,,,,..,:,,.:ft,::,,:,,::.   .fffff1
#                 ,i:,,,,,,,,..,,,,,,;,:::,:,,,::   ;fffff,
#                 .1:,,,,,,,,..,,,:,,,::,,,:,,,,:, .fffff1
#                  ;;:,,,,,:, ..,,,::,,,:,,:,,,,::.,1tfff:
#                  ,1;:,,,,;f:.,,,,,,:,..,,:,..,,::,  ..,
#                   ;i::,,,,,:,,:,,:,:::,,,;1:,.,,,:,
#                   .;;;:,,,,,,,,,::::::::::LGC;,,,,:.
#                    .;;:,,.,,,,,,,,,,,,,,:::;;:,,,,,:
#                     ,:::,.....,,,,,,,,,,,:,,,,,,,,,:.
#                     ....,....,,,,,,.....,,,,,,,,,,,,.
#                     ,,..     .,::;:........,,,,,...
#                     :,,......      ............
#                     :,...........   ....:,...,.
#                     ,,............. ...,1;,,,,.
#                                         ..
