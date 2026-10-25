[CmdletBinding()]
param(
    [ValidateSet('x64','arm64')][string]$Arch = 'x64',
    [string]$Version = '0.1.1'
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$payload = Join-Path $PSScriptRoot 'payload'
$dist = Join-Path $repo 'dist'
$ps = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
function Run-Script([string]$script, [string[]]$args) {
    & $ps -NoProfile -ExecutionPolicy Bypass -File $script @args
    if ($LASTEXITCODE -ne 0) { throw "G7 release step failed: $script ($LASTEXITCODE)" }
}
Run-Script (Join-Path $PSScriptRoot 'build-payload.ps1') @('-Arch',$Arch,'-Version',$Version)
$payloadFiles = @(
    (Join-Path $payload 'clens.exe'),
    (Join-Path $payload 'clens-gui.exe'),
    (Join-Path $payload 'clens-daemon.exe'),
    (Join-Path $payload 'clens-capture.exe')
)
$payloadStatus = Join-Path $dist 'payload-signature-status.json'
$signPayloadArgs = @('-Files') + $payloadFiles + @('-OutputStatus',$payloadStatus)
Run-Script (Join-Path $PSScriptRoot 'sign-release.ps1') $signPayloadArgs
$state = Get-Content -Raw -Encoding utf8 $payloadStatus | ConvertFrom-Json
if (-not $state.formal_release_allowed) { throw 'Payload signing did not produce a Trusted Signing status; formal build is blocked.' }
Run-Script (Join-Path $PSScriptRoot 'build-installer.ps1') @('-Arch',$Arch,'-Version',$Version,'-Release','-Signed')
$installer = Join-Path $dist "CausativeLens-setup-$Arch.exe"
Run-Script (Join-Path $PSScriptRoot 'sign-release.ps1') @('-Files',$installer)
& (Get-Command signtool.exe -ErrorAction Stop) verify /pa /all /v $installer
if ($LASTEXITCODE -ne 0) { throw 'Final installer signature verification failed.' }
Run-Script (Join-Path $PSScriptRoot 'installer-e2e.ps1') @('-InstallerPath',$installer)
Write-Host "Formal G7 release candidate ready: $installer"

