# adding features for developers, used for CI/CD

$file = '.\collapse\static.py'
(Get-Content $file) -replace "DEBUG_LOGS = False", "DEBUG_LOGS = True" | Set-Content $file

$file = '.\collapse\static.py'
(Get-Content $file) -replace "DO_NOT_SAVE_MESSAGES = False", "DO_NOT_SAVE_MESSAGES = True" | Set-Content $file

$file = '.\collapse\static.py'
(Get-Content $file) -replace "SHOW_HIDDEN_MESSAGES = False", "SHOW_HIDDEN_MESSAGES = True" | Set-Content $file

$file = '.\collapse\static.py'
(Get-Content $file) -replace "SHOW_HIDDEN_CHEATS = False", "SHOW_HIDDEN_CHEATS = True" | Set-Content $file

$file = '.\collapse\static.py'
(Get-Content $file) -replace "SKIP_ANIMATIONS = False", "SKIP_ANIMATIONS = True" | Set-Content $file

$file = '.\run.py'
(Get-Content $file) -replace "dev = False", "dev = True" | Set-Content $file