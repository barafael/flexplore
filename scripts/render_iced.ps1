# Renders each testdata case via the Iced golden renderer.
# The binary reads input.json files and saves rendered_iced.png directly.
# Usage: .\scripts\render_iced.ps1 [case1 case2 ...]
param([Parameter(ValueFromRemainingArguments)][string[]]$Cases)

$ErrorActionPreference = 'Stop'
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$TestdataDir = Join-Path (Split-Path -Parent $ScriptDir) "testdata"
$IcedDir = Join-Path $ScriptDir "iced_golden"

Write-Host "=== Building Iced golden renderer ==="
cargo build --release --manifest-path "$IcedDir/Cargo.toml"

Write-Host "=== Rendering Iced screenshots ==="
$exe = Join-Path $IcedDir "target/release/iced-golden.exe"
& $exe $TestdataDir @Cases

Write-Host "All done!"
