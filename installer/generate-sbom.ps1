[CmdletBinding()]
param(
    [string]$Output,
    [string]$Version = '0.2.0'
)
$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrEmpty($Output)) { $Output = Join-Path $PSScriptRoot '..\\dist\\causativelens-sbom.spdx.json' }
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
if (-not (Test-Path -LiteralPath $cargo)) { $cargo = 'cargo.exe' }
Push-Location $repo
try { $raw = & $cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-pc-windows-msvc --features clens-gui/tauri-shell | Out-String; if ($LASTEXITCODE -ne 0) { throw 'cargo metadata failed' } } finally { Pop-Location }
$metadata = $raw | ConvertFrom-Json
$packages = foreach ($p in $metadata.packages) {
    [ordered]@{ SPDXID = ('SPDXRef-Package-' + $p.name.Replace('_','-') + '-' + $p.version); name = $p.name; versionInfo = $p.version; filesAnalyzed=$false; copyrightText='NOASSERTION'; downloadLocation = if ($p.source) { $p.source } else { 'NOASSERTION' }; licenseConcluded = if ($p.license) { $p.license } else { 'NOASSERTION' }; licenseDeclared = if ($p.license) { $p.license } else { 'NOASSERTION' } }
}
$packageIds = @{}
foreach ($package in $metadata.packages) { $packageIds[$package.id] = 'SPDXRef-Package-' + $package.name.Replace('_','-') + '-' + $package.version }
$relationships = foreach ($node in $metadata.resolve.nodes) {
    foreach ($dependency in $node.deps) {
        if ($packageIds.ContainsKey($node.id) -and $packageIds.ContainsKey($dependency.pkg)) {
            [ordered]@{ spdxElementId=$packageIds[$node.id]; relationshipType='DEPENDS_ON'; relatedSpdxElement=$packageIds[$dependency.pkg] }
        }
    }
}
$doc = [ordered]@{
    SPDXID='SPDXRef-DOCUMENT'; spdxVersion='SPDX-2.3'; dataLicense='CC0-1.0'; name="CausativeLens-$Version"; documentNamespace=('https://causativelens.dev/sbom/' + $Version + '/' + [Guid]::NewGuid().ToString()); creationInfo=[ordered]@{ created=[DateTime]::UtcNow.ToString('o'); creators=@('Tool: CausativeLens SBOM generator') }; packages=@($packages); relationships=@($relationships)
}
New-Item -ItemType Directory -Force (Split-Path -Parent $Output) | Out-Null
$doc | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8 $Output
Write-Host "SBOM written: $Output"



