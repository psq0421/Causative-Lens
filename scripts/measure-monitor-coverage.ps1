[CmdletBinding()]
param([double]$Minimum = 80)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
$rustc = Join-Path $env:USERPROFILE '.cargo\bin\rustc.exe'
$sysroot = (& $rustc --print sysroot | Out-String).Trim()
$llvm = Join-Path $sysroot 'lib\rustlib\x86_64-pc-windows-msvc\bin'
if (-not (Test-Path (Join-Path $llvm 'llvm-profdata.exe'))) { throw 'Install rustup component llvm-tools-preview first.' }
$output = Join-Path $repo ('dist\coverage\' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $output -Force | Out-Null
$previousTarget=$env:CARGO_TARGET_DIR; $previousFlags=$env:RUSTFLAGS; $previousProfile=$env:LLVM_PROFILE_FILE
try {
    $env:CARGO_TARGET_DIR = Join-Path $repo 'target\coverage'
    $env:RUSTFLAGS = '-C instrument-coverage'
    $env:LLVM_PROFILE_FILE = Join-Path $output '%p-%m.profraw'
    # Windows PowerShell 5 wraps native stderr in ErrorRecord even for progress.
    $ErrorActionPreference = 'Continue'
    $artifacts = & $cargo test --workspace --tests --offline --message-format=json 2> (Join-Path $output 'build.log')
    $ErrorActionPreference = 'Stop'
    if ($LASTEXITCODE -ne 0) { throw "Instrumented tests failed; see $output\build.log" }
    $executables = @($artifacts | ForEach-Object { try { $item=$_ | ConvertFrom-Json; if ($item.reason -eq 'compiler-artifact' -and $item.profile.test -and $item.executable) { $item.executable } } catch { } } | Sort-Object -Unique)
    $raw = @(Get-ChildItem -LiteralPath $output -Filter '*.profraw' | ForEach-Object FullName)
    if (-not $raw.Count -or -not $executables.Count) { throw 'No coverage inputs were produced.' }
    $profile = Join-Path $output 'coverage.profdata'
    & (Join-Path $llvm 'llvm-profdata.exe') merge -sparse @raw -o $profile
    if ($LASTEXITCODE -ne 0) { throw 'Profile merge failed.' }
    $objects = @($executables[0])
    foreach ($executable in $executables | Select-Object -Skip 1) { $objects += @('-object',$executable) }
    $json = & (Join-Path $llvm 'llvm-cov.exe') export @objects "-instr-profile=$profile" 2> (Join-Path $output 'export.log')
    if ($LASTEXITCODE -ne 0) { throw 'Coverage export failed.' }
    $data = ($json | Out-String) | ConvertFrom-Json
    $files = @($data.data.files | Where-Object { $_.filename -match '[\\/]clens-harnessd[\\/]src[\\/]monitor[\\/]|[\\/]clens-daemon[\\/]src[\\/]monitoring.rs$' })
    $covered = ($files | ForEach-Object { $_.summary.lines.covered } | Measure-Object -Sum).Sum
    $total = ($files | ForEach-Object { $_.summary.lines.count } | Measure-Object -Sum).Sum
    if (-not $total) { throw 'New monitoring modules are missing from coverage output.' }
    $percent = 100 * $covered / $total
    $report = [ordered]@{ scope='new Rust monitoring modules; excludes Tauri native UI'; lines=$total; covered=$covered; percent=[math]::Round($percent,2); minimum=$Minimum; files=@($files | ForEach-Object { [ordered]@{ file=$_.filename; lines=$_.summary.lines } }) }
    $report | ConvertTo-Json -Depth 10 | Set-Content -Encoding utf8 (Join-Path $output 'summary.json')
    $report | ConvertTo-Json -Depth 10
    if ($percent -lt $Minimum) { throw "Monitoring line coverage $percent is below $Minimum percent." }
} finally {
    $env:CARGO_TARGET_DIR=$previousTarget; $env:RUSTFLAGS=$previousFlags; $env:LLVM_PROFILE_FILE=$previousProfile
}
