[CmdletBinding()]
param(
    [ValidateSet('x64','arm64')][string]$Arch = 'x64',
    [string]$Version = '0.1.0',
    [string]$OutputDir,
    [switch]$SkipBuild
)
$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrEmpty($OutputDir)) { $OutputDir = Join-Path $PSScriptRoot 'payload' }
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
if (-not (Test-Path -LiteralPath $cargo)) { $cargo = 'cargo.exe' }
$target = if ($Arch -eq 'arm64') { 'aarch64-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }
$targetArgs = @('--target', $target)
if (-not $SkipBuild) {
    Push-Location $repo
    try {
        & $cargo build --release @targetArgs -p clens-cli -p clens-daemon -p clens-capture
        if ($LASTEXITCODE -ne 0) { throw "Rust core build failed: $LASTEXITCODE" }
        & $cargo build --release @targetArgs -p clens-gui --features tauri-shell
        if ($LASTEXITCODE -ne 0) { throw "Tauri GUI build failed: $LASTEXITCODE" }
    } finally { Pop-Location }
}
$binRoot = Join-Path $repo ("target\$target\release")
# Cargo keeps the package name (`clens-cli.exe`) on disk; the installer
# renames it to the public `clens.exe` entry point below. Do not accidentally
# fall back to an older host build when the cross-target directory is fresh.
if (-not (Test-Path (Join-Path $binRoot 'clens-cli.exe'))) {
    $binRoot = Join-Path $repo 'target\release'
}
$required = @('clens.exe','clens-gui.exe','clens-daemon.exe','clens-capture.exe')
$sources = @{ 'clens.exe' = 'clens-cli.exe'; 'clens-gui.exe' = 'clens-gui.exe'; 'clens-daemon.exe' = 'clens-daemon.exe'; 'clens-capture.exe' = 'clens-capture.exe' }
foreach ($name in $required) {
    $source = Join-Path $binRoot $sources[$name]
    if (-not (Test-Path -LiteralPath $source)) { throw "Missing release binary: $source" }
}
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
foreach ($name in $required) { Copy-Item -Force (Join-Path $binRoot $sources[$name]) (Join-Path $OutputDir $name) }
Copy-Item -Force (Join-Path $repo 'crates\clens-capture\src\clens-capture.manifest') (Join-Path $OutputDir 'clens-capture.exe.manifest')
Copy-Item -Force (Join-Path $repo 'crates\clens-gui\icons\icon.ico') (Join-Path $OutputDir 'causativelens.ico')
$cliVersion = (& (Join-Path $OutputDir 'clens.exe') --version | Out-String).Trim()
$guiVersion = (& (Join-Path $OutputDir 'clens-gui.exe') --version | Out-String).Trim()
if ($cliVersion -notmatch [regex]::Escape($Version)) { throw "CLI version '$cliVersion' does not contain $Version" }
if ($guiVersion -notmatch [regex]::Escape($Version)) { throw "GUI version '$guiVersion' does not contain $Version" }
$files = foreach ($name in ($required + @('clens-capture.exe.manifest','causativelens.ico'))) {
    $path = Join-Path $OutputDir $name
    $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant()
    [ordered]@{ name = $name; sha256 = $hash; bytes = (Get-Item -LiteralPath $path).Length }
}
$manifest = [ordered]@{
    schema = 'causativelens.payload/1'
    product = 'CausativeLens'
    version = $Version
    architecture = $Arch
    mode = 'observe'
    cli_version = $cliVersion
    gui_version = $guiVersion
    files = @($files)
    generated_utc = [DateTime]::UtcNow.ToString('o')
}
$manifest | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 (Join-Path $OutputDir 'payload-manifest.json')
Write-Host "Payload ready: $OutputDir ($Arch, $Version)"



