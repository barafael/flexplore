# Re-generates all rendered images and the HTML overview.
# Usage: .\scripts\render_all.ps1 [case1 case2 ...]
#   No args = all cases. Pass case names to render a subset.
param([Parameter(ValueFromRemainingArguments)][string[]]$Cases)

$ErrorActionPreference = 'Stop'
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir

Push-Location $RootDir
try {
    Write-Host "=== Regenerating snapshot test data ==="
    $env:FLEXPLORE_UPDATE = "1"
    cargo test snapshot -- --test-threads=1
    Remove-Item Env:\FLEXPLORE_UPDATE

    Write-Host "`n=== Rendering Bevy screenshots ==="
    cargo run --example render_testdata -- @Cases

    Write-Host "`n=== Rendering HTML screenshots ==="
    & "$ScriptDir/render_html.ps1" @Cases

    Write-Host "`n=== Rendering Tailwind screenshots ==="
    & "$ScriptDir/render_tailwind.ps1" @Cases

    Write-Host "`n=== Rendering Flutter screenshots ==="
    & "$ScriptDir/render_flutter.ps1" @Cases

    Write-Host "`n=== Rendering Swift screenshots ==="
    & "$ScriptDir/render_swift.ps1" @Cases

    Write-Host "`n=== Building overview page ==="
    python3 "$ScriptDir/build_overview.py"

    Write-Host "`nDone! Open testdata/overview.html to compare."
} finally {
    Pop-Location
}
