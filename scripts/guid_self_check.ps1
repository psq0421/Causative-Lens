$ErrorActionPreference = 'Stop'
$expected = @{
  'Process' = '22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716'
  'File' = 'edd08927-9cc4-4e65-b970-c2560fb5c289'
  'Registry' = '70eb4f03-c1de-4f73-a051-33d13d5413bd'
  'Network' = '7dd42a49-5329-4832-8dfd-43d979153a88'
}
$guidFile = Join-Path $PSScriptRoot '..\docs\spec\etw-guids.json'
if (-not (Test-Path $guidFile)) {
  Write-Error "Missing ETW GUID contract: $guidFile"
}
$actual = Get-Content -Raw $guidFile | ConvertFrom-Json
foreach ($key in $expected.Keys) {
  if (-not ($actual.PSObject.Properties.Name -contains $key)) { Write-Error "Missing provider $key" }
  if ([string]::IsNullOrWhiteSpace($actual.$key)) { Write-Error "Empty GUID for $key" }
  [guid]$actual.$key | Out-Null
  if ($actual.$key.ToLowerInvariant() -ne $expected[$key]) { Write-Error "GUID mismatch for ${key}: expected $($expected[$key]), got $($actual.$key)" }
}
Write-Output 'ETW GUID contract is syntactically valid.'
