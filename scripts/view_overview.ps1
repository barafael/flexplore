# Regenerate all golden files, render every backend, build the HTML overview.
# Usage: .\scripts\view_overview.ps1 [case1 case2 ...]
param([Parameter(ValueFromRemainingArguments)][string[]]$Cases)

$ErrorActionPreference = 'Stop'
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir

Push-Location $RootDir
try {
    Write-Host ">>> Regenerating golden files"
    cargo run -p update-snapshots
    if ($LASTEXITCODE -ne 0) { throw "update-snapshots failed" }

    Write-Host ">>> Rendering Bevy screenshots"
    if ($Cases) {
        cargo run --example render_testdata -- @Cases
    } else {
        cargo run --example render_testdata
    }
    if ($LASTEXITCODE -ne 0) { throw "Bevy render failed" }

    Write-Host ">>> Rendering HTML screenshots"
    bash "$ScriptDir/render_html.sh" @Cases
    if ($LASTEXITCODE -ne 0) { throw "HTML render failed" }

    Write-Host ">>> Rendering Tailwind screenshots"
    bash "$ScriptDir/render_tailwind.sh" @Cases
    if ($LASTEXITCODE -ne 0) { throw "Tailwind render failed" }

    Write-Host ">>> Rendering Flutter screenshots"
    if (Get-Command flutter -ErrorAction SilentlyContinue) {
        bash "$ScriptDir/render_flutter.sh" @Cases
        if ($LASTEXITCODE -ne 0) { throw "Flutter render failed" }
    } else {
        Write-Host "  SKIP: flutter not found"
    }

    Write-Host ">>> Rendering Swift screenshots"
    if (Get-Command swift -ErrorAction SilentlyContinue) {
        bash "$ScriptDir/render_swift.sh" @Cases
        if ($LASTEXITCODE -ne 0) { throw "Swift render failed" }
    } else {
        Write-Host "  SKIP: swift not found (requires macOS)"
    }

    Write-Host ">>> Rendering Iced screenshots"
    bash "$ScriptDir/render_iced.sh" @Cases
    if ($LASTEXITCODE -ne 0) { throw "Iced render failed" }

    Write-Host ">>> Building overview page"
    python "$ScriptDir/build_overview.py"

    Write-Host ""
    Write-Host "Done! Open testdata/overview.html to view the comparison."
} finally {
    Pop-Location
}
