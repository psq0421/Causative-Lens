[CmdletBinding()]
param(
    [ValidateSet('x64','arm64')][string]$Arch = 'x64',
    [string]$Version = '0.2.0',
    [string]$PayloadDir,
    [string]$DistDir,
    [switch]$Release,
    [switch]$Signed
)
$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrEmpty($PayloadDir)) { $PayloadDir = Join-Path $PSScriptRoot 'payload' }
if ([string]::IsNullOrEmpty($DistDir)) { $DistDir = Join-Path (Join-Path $PSScriptRoot '..') 'dist' }
if ($Release -and -not $Signed) { throw 'A formal release installer requires -Signed after Trusted Signing verification.' }
if ($Release) {
    $payloadStatusPath = Join-Path (Join-Path $PSScriptRoot '..') 'dist\payload-signature-status.json'
    if (-not (Test-Path -LiteralPath $payloadStatusPath)) { throw 'Formal installer requires a verified payload-signature-status.json.' }
    $payloadStatus = Get-Content -Raw -Encoding utf8 $payloadStatusPath | ConvertFrom-Json
    if (-not $payloadStatus.formal_release_allowed) { throw 'Payload signature status is not Azure Trusted Signing; formal installer is blocked.' }
}
$nsis = 'C:\Program Files (x86)\NSIS\makensis.exe'
if (-not (Test-Path -LiteralPath $nsis)) { throw "NSIS compiler not found: $nsis" }
$manifestPath = Join-Path $PayloadDir 'payload-manifest.json'
if (-not (Test-Path -LiteralPath $manifestPath)) { throw "Payload manifest missing: $manifestPath" }
$manifest = Get-Content -Raw -Encoding utf8 $manifestPath | ConvertFrom-Json
if ($manifest.version -ne $Version -or $manifest.architecture -ne $Arch) { throw 'Payload manifest version/architecture does not match installer request.' }
if ($Release -and $manifest.cli_version -notmatch [regex]::Escape($Version)) { throw 'CLI is not a release-version payload.' }
if ($Release -and $manifest.gui_version -notmatch [regex]::Escape($Version)) { throw 'GUI is not a release-version payload.' }
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null
$name = if ($Release) { "CausativeLens-setup-$Arch.exe" } else { "CausativeLens-setup-$Arch-dev.exe" }
$out = Join-Path $DistDir $name
$payloadRelative = Split-Path -Leaf $PayloadDir
Push-Location $PSScriptRoot
try {
    & $nsis "/DPRODUCT_VERSION=$Version" "/DARCH=$Arch" "/DPAYLOAD_DIR=$payloadRelative" "/DOUT_FILE=$out" 'causativelens.nsi'
    if ($LASTEXITCODE -ne 0) { throw "NSIS compilation failed: $LASTEXITCODE" }
} finally { Pop-Location }
if (-not (Test-Path -LiteralPath $out)) { throw "NSIS did not produce $out" }
$item = Get-Item -LiteralPath $out
if ($item.Length -ge 25MB) { throw "Installer is $($item.Length) bytes; the 25MB red line was exceeded." }
$hash = Get-FileHash -Algorithm SHA256 -LiteralPath $out
$hash | ConvertTo-Json | Set-Content -Encoding utf8 ($out + '.sha256.json')
Write-Host "Installer ready: $out ($($item.Length) bytes)"




