# Renders each testdata/*/expected.html to testdata/*/rendered_html.png
# Usage: .\scripts\render_html.ps1 [case1 case2 ...]
param([Parameter(ValueFromRemainingArguments)][string[]]$Cases)

$ErrorActionPreference = 'Stop'
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$TestdataDir = Join-Path (Split-Path -Parent $ScriptDir) "testdata"
$Viewport = "400,300"

if (-not $Cases) {
    $Cases = Get-ChildItem -Directory $TestdataDir | ForEach-Object { $_.Name }
}

Write-Host "Will render $($Cases.Count) HTML test case(s)"

foreach ($name in $Cases) {
    $htmlFile = Join-Path $TestdataDir "$name/expected.html"
    $outFile = Join-Path $TestdataDir "$name/rendered_html.png"

    if (-not (Test-Path $htmlFile)) {
        Write-Host "  SKIP: $htmlFile not found"
        continue
    }

    $fileUrl = "file:///$($htmlFile -replace '\\','/')"

    npx playwright screenshot `
        --browser chromium `
        --viewport-size="$Viewport" `
        $fileUrl $outFile *>$null

    Write-Host "  Saved: $name/rendered_html.png"
}

Write-Host "All done!"
