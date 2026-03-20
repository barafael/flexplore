# Renders each testdata/*/expected.tailwind.html to testdata/*/rendered_tailwind.png
# Wraps the fragment in a full HTML page with the Tailwind CDN, writes a temp file,
# then screenshots it with Playwright.
# Usage: .\scripts\render_tailwind.ps1 [case1 case2 ...]
param([Parameter(ValueFromRemainingArguments)][string[]]$Cases)

$ErrorActionPreference = 'Stop'
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$TestdataDir = Join-Path (Split-Path -Parent $ScriptDir) "testdata"
$Viewport = "400,300"

if (-not $Cases) {
    $Cases = Get-ChildItem -Directory $TestdataDir | ForEach-Object { $_.Name }
}

Write-Host "Will render $($Cases.Count) Tailwind test case(s)"

$header = @"
<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<script src="https://cdn.tailwindcss.com"></script>
<style>html,body{margin:0;height:100%}body{display:flex;flex-direction:column;align-items:flex-start}</style>
</head>
<body>
"@

$footer = @"
</body>
</html>
"@

foreach ($name in $Cases) {
    $twFile = Join-Path $TestdataDir "$name/expected.tailwind.html"
    $outFile = Join-Path $TestdataDir "$name/rendered_tailwind.png"
    $tmpFile = Join-Path $TestdataDir "$name/_tmp_tailwind.html"

    if (-not (Test-Path $twFile)) {
        Write-Host "  SKIP: $twFile not found"
        continue
    }

    $body = Get-Content -Raw $twFile
    ($header + "`n" + $body + "`n" + $footer) | Set-Content -Encoding UTF8 $tmpFile

    $fileUrl = "file:///$($tmpFile -replace '\\','/')"

    npx playwright screenshot `
        --browser chromium `
        --viewport-size="$Viewport" `
        --wait-for-timeout=2000 `
        $fileUrl $outFile *>$null

    Remove-Item $tmpFile -ErrorAction SilentlyContinue
    Write-Host "  Saved: $name/rendered_tailwind.png"
}

Write-Host "All done!"
