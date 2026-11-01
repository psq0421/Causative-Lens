[CmdletBinding()]
param(
    [string]$Repository = (Get-Location).Path
)
$ErrorActionPreference = 'Stop'
Push-Location $Repository
try {
    $files = @(git -c core.quotePath=false ls-files | Where-Object { $_ -ne 'scripts/count-code-lines.ps1' })
    $groups = [ordered]@{
        RustProduction = '^crates/.+/(src/.+|build)\.rs$'
        RustTestsBenchesFuzz = '^crates/.+/(tests|benches)/.+\.rs$|^fuzz/.+\.rs$'
        Frontend = '^crates/clens-gui/frontend/.+\.(js|mjs|html|css)$|^tests/gui/.+\.(js|mjs|html|css)$'
        PackagingAndScripts = '^(installer|scripts|docs/guide/examples)/.+\.(ps1|nsi)$'
        Workflows = '^\.github/workflows/.+\.(yml|yaml)$'
    }
    $rows = @()
    foreach ($entry in $groups.GetEnumerator()) {
        $selected = @($files | Where-Object { $_ -match $entry.Value })
        $physical = 0
        $nonEmpty = 0
        foreach ($path in $selected) {
            $lines = @(Get-Content -LiteralPath $path -Encoding utf8)
            $physical += $lines.Count
            foreach ($line in $lines) {
                if ($line.Trim().Length -gt 0) { $nonEmpty++ }
            }
        }
        $rows += [pscustomobject]@{
            group = $entry.Key
            files = $selected.Count
            physical_lines = $physical
            non_empty_lines = $nonEmpty
        }
    }
    $executable = $rows | Where-Object { $_.group -ne 'Workflows' }
    $all = [pscustomobject]@{
        schema = 'causativelens.code-size/1'
        commit = (git rev-parse HEAD)
        measured_at = [DateTime]::UtcNow.ToString('o')
        scope = 'tracked files only; docs, Cargo metadata, JSON data, generated target/dist, and .git excluded'
        groups = $rows
        executable_physical_lines = ($executable | Measure-Object -Property physical_lines -Sum).Sum
        executable_non_empty_lines = ($executable | Measure-Object -Property non_empty_lines -Sum).Sum
        code_plus_ci_physical_lines = ($rows | Measure-Object -Property physical_lines -Sum).Sum
        code_plus_ci_non_empty_lines = ($rows | Measure-Object -Property non_empty_lines -Sum).Sum
    }
    $all | ConvertTo-Json -Depth 8
} finally {
    Pop-Location
}

