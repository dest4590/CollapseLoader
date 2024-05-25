
# unremoves features for developers, used for CI/CD

$file = '.\collapse\utils\Logger.py'
$regex = "logger = setup_logger\('CollapseLogger', logging\.INFO\)"
(Get-Content $file) -replace $regex, "logger = setup_logger('CollapseLogger', logging.DEBUG)" | Set-Content $file

$file = '.\run.py'
$regex = "dev = False"
(Get-Content $file) -replace $regex, "dev = True" | Set-Content $file