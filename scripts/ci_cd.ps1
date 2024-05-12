
# removes features for developers, used for CI/CD

$file = '.\collapse\utils\Logger.py'
$regex = "logger = setup_logger\('CollapseLogger', logging\.DEBUG\)"
(Get-Content $file) -replace $regex, "logger = setup_logger('CollapseLogger', logging.INFO)" | Set-Content $file

$file = '.\run.py'
$regex = "dev = True"
(Get-Content $file) -replace $regex, "dev = False" | Set-Content $file