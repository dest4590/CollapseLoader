
# removes features for developers, used for CI/CD

$file = '.\collapse\static.py'
(Get-Content $file) -replace "DEBUG_LOGS = True", "DEBUG_LOGS = False" | Set-Content $file


$file = '.\collapse\static.py'
(Get-Content $file) -replace "LOCAL_API = True", "LOCAL_API = False" | Set-Content $file

$file = '.\collapse\static.py'
(Get-Content $file) -replace "DO_NOT_SAVE_MESSAGES = True", "DO_NOT_SAVE_MESSAGES = False" | Set-Content $file

$file = '.\run.py'
(Get-Content $file) -replace "dev = True", "dev = False" | Set-Content $file