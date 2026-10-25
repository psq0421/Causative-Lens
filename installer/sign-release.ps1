[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string[]]$Files,
    [string]$OutputStatus,
    [switch]$AllowDevUnsigned
)
$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrEmpty($OutputStatus)) { $OutputStatus = Join-Path $PSScriptRoot '..\\dist\\signature-status.json' }
$signtool = (Get-Command signtool.exe -ErrorAction SilentlyContinue).Source
if (-not $signtool) {
    $sdk = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin' -Filter signtool.exe -Recurse -ErrorAction SilentlyContinue | Sort-Object FullName -Descending | Select-Object -First 1
    if ($sdk) { $signtool = $sdk.FullName }
}
$dlib = $env:AZURE_TRUSTED_SIGNING_DLIB
$metadata = $env:AZURE_TRUSTED_SIGNING_METADATA
$hasConfig = $signtool -and $dlib -and (Test-Path -LiteralPath $dlib) -and $metadata -and (Test-Path -LiteralPath $metadata)
if (-not $hasConfig) {
    $status = [ordered]@{ schema='causativelens.signature/1'; status='unsigned-dev'; formal_release_allowed=$false; checked_utc=[DateTime]::UtcNow.ToString('o'); reason='Azure Trusted Signing configuration is incomplete; no self-signed fallback is permitted.'; files=@($Files) }
    New-Item -ItemType Directory -Force (Split-Path -Parent $OutputStatus) | Out-Null
    $status | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 $OutputStatus
    if (-not $AllowDevUnsigned) { throw 'Trusted Signing is not configured. Formal release is blocked; use -AllowDevUnsigned only for a -dev package.' }
    Write-Warning 'Unsigned development status written; this package must not be uploaded to GitHub Releases.'
    return
}
foreach ($file in $Files) {
    if (-not (Test-Path -LiteralPath $file)) { throw "Signing input missing: $file" }
    & $signtool sign /v /fd SHA256 /tr http://timestamp.acs.microsoft.com /td SHA256 /dlib $dlib /dmdf $metadata $file
    if ($LASTEXITCODE -ne 0) { throw "Trusted Signing failed for $file ($LASTEXITCODE)" }
    & $signtool verify /pa /all /v $file
    if ($LASTEXITCODE -ne 0) { throw "Authenticode verification failed for $file ($LASTEXITCODE)" }
}
$status = [ordered]@{ schema='causativelens.signature/1'; status='azure-trusted-signing'; formal_release_allowed=$true; checked_utc=[DateTime]::UtcNow.ToString('o'); files=@($Files); signer='Azure Trusted Signing'; timestamp='http://timestamp.acs.microsoft.com' }
New-Item -ItemType Directory -Force (Split-Path -Parent $OutputStatus) | Out-Null
$status | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 $OutputStatus
Write-Host 'All files carry a verified Azure Trusted Signing signature.'


