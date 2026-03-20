# Renders each testdata case via SwiftUI snapshot tests on macOS, then copies
# the resulting PNGs into testdata/*/rendered_swift.png.
# Usage: .\scripts\render_swift.ps1 [case1 case2 ...]
param([Parameter(ValueFromRemainingArguments)][string[]]$Cases)

$ErrorActionPreference = 'Stop'
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$TestdataDir = Join-Path (Split-Path -Parent $ScriptDir) "testdata"
$SwiftDir = Join-Path $ScriptDir "swift_golden"
$SnapshotsDir = Join-Path $SwiftDir "Tests/SwiftGoldenTests/__Snapshots__/GoldenTests"

if (-not (Get-Command swift -ErrorAction SilentlyContinue)) {
    Write-Error "swift not found in PATH (requires macOS with Xcode CLI tools)"
    exit 1
}

Write-Host "=== Regenerating Swift view files ==="
python3 "$SwiftDir/tool/generate_cases.py"

Write-Host "=== Running Swift snapshot tests ==="
Push-Location $SwiftDir
try {
    $env:SWIFT_SNAPSHOT_RECORD = "1"
    # Record mode always "fails" tests (by design) — ignore the exit code.
    swift test 2>&1 | Out-Host
    Remove-Item Env:\SWIFT_SNAPSHOT_RECORD
} finally {
    Pop-Location
}

Write-Host "=== Copying snapshots to testdata ==="
if (-not $Cases) {
    $Cases = Get-ChildItem "$SnapshotsDir/test_*.1.png" -ErrorAction SilentlyContinue |
        ForEach-Object { $_.BaseName -replace '^test_(.*)\.1$','$1' }
}

foreach ($name in $Cases) {
    $src = Join-Path $SnapshotsDir "test_$name.1.png"
    $dst = Join-Path $TestdataDir "$name/rendered_swift.png"
    if ((Test-Path $src) -and (Test-Path (Join-Path $TestdataDir $name))) {
        Copy-Item $src $dst
        Write-Host "  Copied: $name/rendered_swift.png"
    }
}

Write-Host "All done!"
