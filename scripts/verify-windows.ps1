$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
New-Item -ItemType Directory -Force -Path artifacts | Out-Null
$brushLogPath = Join-Path (Get-Location) ('artifacts/windows-check-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.log')

function Invoke-BrushCheck {
    param([string]$Program, [string[]]$Arguments)
    Write-Host "Running: $Program $Arguments"
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed with exit code $LASTEXITCODE" }
}

Start-Transcript -Path $brushLogPath
try {
    Get-ComputerInfo -Property WindowsProductName, WindowsVersion, OsBuildNumber, CsSystemType
    Invoke-BrushCheck 'node' @('--version')
    Invoke-BrushCheck 'rustc' @('--version')
    Invoke-BrushCheck 'npm.cmd' @('ci')
    Invoke-BrushCheck 'npm.cmd' @('test')
    Invoke-BrushCheck 'cargo' @('fmt', '--manifest-path', 'src-tauri/Cargo.toml', '--', '--check')
    Invoke-BrushCheck 'cargo' @('test', '--locked', '--manifest-path', 'src-tauri/Cargo.toml')
    Invoke-BrushCheck 'cargo' @('clippy', '--locked', '--manifest-path', 'src-tauri/Cargo.toml', '--all-targets', '--', '-D', 'warnings')
    Invoke-BrushCheck 'npm.cmd' @('run', 'tauri', 'build', '--', '--bundles', 'nsis')
    Write-Host 'Build checks passed. Install the generated NSIS executable, then follow docs/testing/compatibility.md.'
    Write-Host 'A successful build does not validate native drawing, Korean IME, DPI, or screen sharing.'
} finally {
    Stop-Transcript
    Write-Host "Log: $brushLogPath"
}
