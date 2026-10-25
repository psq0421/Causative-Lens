[CmdletBinding()]
param(
    [string]$Output,
    [string]$Version = '0.1.0'
)
$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrEmpty($Output)) { $Output = Join-Path $PSScriptRoot '..\\dist\\causativelens-sbom.spdx.json' }
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
if (-not (Test-Path -LiteralPath $cargo)) { $cargo = 'cargo.exe' }
Push-Location $repo
try { $raw = & $cargo metadata --locked --format-version 1 --no-deps | Out-String; if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed' } } finally { Pop-Location }
$metadata = $raw | ConvertFrom-Json
$packages = foreach ($p in $metadata.packages) {
    [ordered]@{ SPDXID = ('SPDXRef-Package-' + $p.name.Replace('_','-')); name = $p.name; versionInfo = $p.version; downloadLocation = 'NOASSERTION'; licenseConcluded = if ($p.license) { $p.license } else { 'NOASSERTION' }; licenseDeclared = if ($p.license) { $p.license } else { 'NOASSERTION' } }
}
$doc = [ordered]@{
    SPDXID='SPDXRef-DOCUMENT'; spdxVersion='SPDX-2.3'; dataLicense='CC0-1.0'; name="CausativeLens-$Version"; documentNamespace=('https://causativelens.dev/sbom/' + $Version + '/' + [Guid]::NewGuid().ToString()); creationInfo=[ordered]@{ created=[DateTime]::UtcNow.ToString('o'); creators=@('Tool: CausativeLens SBOM generator') }; packages=@($packages)
}
New-Item -ItemType Directory -Force (Split-Path -Parent $Output) | Out-Null
$doc | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8 $Output
Write-Host "SBOM written: $Output"


