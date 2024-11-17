$files = @(
    '.\collapse\developer.py',
    '.\run.py'
)

$replacements = @(
    "DEBUG_LOGS = True", "DEBUG_LOGS = False",
    "DO_NOT_SAVE_MESSAGES = True", "DO_NOT_SAVE_MESSAGES = False",
    "SHOW_HIDDEN_CLIENTS = True", "SHOW_HIDDEN_CLIENTS = False",
    "LOCAL_API = True", "LOCAL_API = False",
    "SAVE_MESSAGES = False", "SAVE_MESSAGES = True",
    "dev = True", "dev = False"
)

$replacementCount = $replacements.Length / 2

foreach ($file in $files) {
    for ($i = 0; $i -lt $replacementCount; $i++) {
        (Get-Content $file) -replace $replacements[$i * 2], $replacements[$i * 2 + 1] | Set-Content $file
    }
}