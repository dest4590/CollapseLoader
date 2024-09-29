$files = @(
    '.\collapse\developer.py',
    '.\run.py'
)

$replacements = @(
    "DEBUG_LOGS = False", "DEBUG_LOGS = True",
    "SKIP_ANIMATIONS = False", "SKIP_ANIMATIONS = True",
    "dev = False", "dev = True"
)

$replacementCount = $replacements.Length / 2

foreach ($file in $files) {
    for ($i = 0; $i -lt $replacementCount; $i++) {
        (Get-Content $file) -replace $replacements[$i * 2], $replacements[$i * 2 + 1] | Set-Content $file
    }
}