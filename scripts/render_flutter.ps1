# Renders each testdata case via Flutter golden tests, then copies the
# resulting PNGs into testdata/*/rendered_flutter.png.
# Usage: .\scripts\render_flutter.ps1 [case1 case2 ...]
param([Parameter(ValueFromRemainingArguments)][string[]]$Cases)

$ErrorActionPreference = 'Stop'
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$TestdataDir = Join-Path (Split-Path -Parent $ScriptDir) "testdata"
$FlutterDir = Join-Path $ScriptDir "flutter_golden"
$GoldensDir = Join-Path $FlutterDir "test/goldens"

if (-not (Get-Command flutter -ErrorAction SilentlyContinue)) {
    Write-Error "flutter not found in PATH"
    exit 1
}

# Copy Roboto from Flutter SDK cache so golden tests render real text
$flutterBin = (Get-Command flutter).Source
$flutterRoot = Split-Path -Parent (Split-Path -Parent $flutterBin)
$fontSrc = Join-Path $flutterRoot "bin/cache/artifacts/material_fonts/roboto-regular.ttf"
$fontDst = Join-Path $FlutterDir "fonts/Roboto-Regular.ttf"
if ((Test-Path $fontSrc) -and -not (Test-Path $fontDst)) {
    New-Item -ItemType Directory -Path (Split-Path -Parent $fontDst) -Force | Out-Null
    Copy-Item $fontSrc $fontDst
    Write-Host "  Copied Roboto font from Flutter SDK"
}

Write-Host "=== Regenerating Flutter widget files ==="
python3 "$FlutterDir/tool/generate_cases.py"

Write-Host "=== Running Flutter golden tests ==="
Push-Location $FlutterDir
try {
    flutter test --update-goldens test/golden_test.dart
} finally {
    Pop-Location
}

Write-Host "=== Copying goldens to testdata ==="
if (-not $Cases) {
    $Cases = Get-ChildItem "$GoldensDir/*.png" | ForEach-Object { $_.BaseName }
}

foreach ($name in $Cases) {
    $src = Join-Path $GoldensDir "$name.png"
    $dst = Join-Path $TestdataDir "$name/rendered_flutter.png"
    if ((Test-Path $src) -and (Test-Path (Join-Path $TestdataDir $name))) {
        Copy-Item $src $dst
        Write-Host "  Copied: $name/rendered_flutter.png"
    }
}

Write-Host "All done!"
