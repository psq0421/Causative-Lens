$ErrorActionPreference = 'Stop'

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw 'cargo is required for dependency audit.'
}

if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
  cargo deny check advisories licenses bans sources
} else {
  Write-Warning 'cargo-deny is not installed; running cargo metadata and cargo audit fallback.'
  cargo metadata --locked --format-version 1 --no-deps | Out-Null
  if (Get-Command cargo-audit -ErrorAction SilentlyContinue) {
    cargo audit --locked
  } else {
    Write-Warning 'cargo-audit is not installed; CI must install cargo-deny and cargo-audit before release.'
  }
}
