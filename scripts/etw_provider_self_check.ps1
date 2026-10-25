$ErrorActionPreference = 'Stop'
$contract = Get-Content -Raw (Join-Path $PSScriptRoot '..\docs\spec\etw-guids.json') | ConvertFrom-Json
$providers = @{
  Process = 'Microsoft-Windows-Kernel-Process'
  File = 'Microsoft-Windows-Kernel-File'
  Registry = 'Microsoft-Windows-Kernel-Registry'
  Network = 'Microsoft-Windows-Kernel-Network'
}
foreach ($key in $providers.Keys) {
  $name = $providers[$key]
  $line = (logman query providers $name | Select-String -Pattern $name | Select-Object -First 1).ToString()
  if ($line -notmatch '\{([0-9A-Fa-f-]{36})\}') { throw "logman did not return GUID for $name" }
  $actual = $Matches[1].ToLowerInvariant()
  if ($actual -ne $contract.$key.ToLowerInvariant()) { throw "GUID mismatch for ${key}: $actual" }
  Write-Output "$key $actual OK"
}
