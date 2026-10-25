$ErrorActionPreference = 'Stop'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Run this smoke test from an elevated clens-capture process.' }
$name = 'CausativeLens_G1Smoke'
$etl = Join-Path $env:TEMP "$name.etl"
logman stop $name -ets 2>$null | Out-Null
logman delete $name 2>$null | Out-Null
logman create trace $name -o $etl -p Microsoft-Windows-Kernel-Process 0xFFFFFFFFFFFFFFFF -ets | Out-Host
Start-Sleep -Seconds 2
logman stop $name -ets | Out-Host
logman delete $name | Out-Host
if (-not (Test-Path $etl)) { throw "ETL file was not produced: $etl" }
$file = Get-Item $etl
Write-Output "ETW smoke passed: $($file.FullName), $($file.Length) bytes"
Remove-Item $etl -Force
