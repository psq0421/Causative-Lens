[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$InstallerPath,
    [string]$InstallDir = (Join-Path $env:TEMP ("CausativeLens E2E 中文 " + [Guid]::NewGuid().ToString('N'))),
    [switch]$KeepInstall,
    [switch]$SkipGuiColdStart
)
$ErrorActionPreference = 'Stop'
$InstallerPath = (Resolve-Path -LiteralPath $InstallerPath).Path
$InstallDir = [IO.Path]::GetFullPath($InstallDir)
$log = Join-Path $env:TEMP ("causativelens-e2e-" + [Guid]::NewGuid().ToString('N') + '.log')
$report = [ordered]@{ schema='causativelens.installer-e2e/1'; installer=$InstallerPath; install_dir=$InstallDir; started_utc=[DateTime]::UtcNow.ToString('o'); steps=@() }
function Assert-That([bool]$condition, [string]$message) { if (-not $condition) { throw $message } }
function Invoke-Installer([string[]]$arguments, [int[]]$allowed=@(0,3010)) {
    $p = Start-Process -FilePath $InstallerPath -ArgumentList $arguments -Wait -PassThru -WindowStyle Hidden
    Assert-That ($allowed -contains $p.ExitCode) "Installer exit code $($p.ExitCode) was not in [$($allowed -join ',')]"
    $report.steps += [ordered]@{ action='installer'; args=($arguments -join ' '); exit_code=$p.ExitCode }
}
function Assert-RegistryAndShell {
    Assert-That (Test-Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\CausativeLens") 'Uninstall registry key missing'
    $app = Get-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\CausativeLens"
    Assert-That ($app.DisplayVersion -eq '0.1.0') "DisplayVersion is $($app.DisplayVersion)"
    Assert-That (Test-Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\App Paths\clens.exe") 'CLI App Paths key missing'
    Assert-That (Test-Path 'HKCU:\Software\Classes\.cllens') '.cllens association missing'
    Assert-That (Test-Path (Join-Path ([Environment]::GetFolderPath('Desktop')) 'CausativeLens.lnk')) 'Desktop shortcut missing'
    Assert-That (Test-Path (Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\CausativeLens\CausativeLens GUI.lnk')) 'Start Menu GUI shortcut missing'
}
try {
    if (Test-Path -LiteralPath $InstallDir) { Remove-Item -LiteralPath $InstallDir -Recurse -Force }
    New-Item -ItemType Directory -Force -Path (Split-Path $InstallDir) | Out-Null
    $installArgs = @('/S','/NORESTART',"/LOG=$log", "/D=$InstallDir")
    Invoke-Installer $installArgs
    foreach ($file in @('clens.exe','clens-gui.exe','clens-daemon.exe','clens-capture.exe','uninstall.exe')) { Assert-That (Test-Path (Join-Path $InstallDir $file)) "Installed file missing: $file" }
    $env:Path = "$InstallDir;$env:Path"
    $cliVersion = (& (Join-Path $InstallDir 'clens.exe') --version | Out-String).Trim()
    $guiVersion = (& (Join-Path $InstallDir 'clens-gui.exe') --version | Out-String).Trim()
    Assert-That ($cliVersion -match '0\.1\.0') "CLI version mismatch: $cliVersion"
    Assert-That ($guiVersion -match '0\.1\.0') "GUI version mismatch: $guiVersion"
    $doctor = (& (Join-Path $InstallDir 'clens.exe') doctor --json | Out-String).Trim()
    Assert-That ($LASTEXITCODE -eq 0) "clens doctor failed: $doctor"
    Assert-That ($doctor -match 'observe') "doctor is not observe-only: $doctor"
    $englishHelp = (& (Join-Path $InstallDir 'clens.exe') --lang en-US --help | Out-String).Trim()
    $simplifiedHelp = (& (Join-Path $InstallDir 'clens.exe') --lang zh-CN --help | Out-String).Trim()
    $traditionalDisabled = (& (Join-Path $InstallDir 'clens.exe') --lang zh-TW advise 2>&1 | Out-String).Trim()
    Assert-That ($englishHelp -match 'Usage:') 'English CLI locale did not render'
    $simplifiedMarker = ([char]0x7528).ToString() + ([char]0x6cd5).ToString()
    $traditionalMarker = ([char]0x50c5).ToString() + ([char]0x652f).ToString() + ([char]0x63f4).ToString() + ' observe'
    Assert-That ($simplifiedHelp.Contains($simplifiedMarker)) 'Simplified Chinese CLI locale did not render'
    Assert-That ($traditionalDisabled.Contains($traditionalMarker)) 'Traditional Chinese CLI locale did not render'
    $userPathAfterInstall = (Get-ItemProperty -Path 'HKCU:\Environment' -Name Path -ErrorAction SilentlyContinue).Path
    Assert-That (($userPathAfterInstall -split ';') -contains $InstallDir) 'User PATH was not injected idempotently'
    Assert-RegistryAndShell
    $report.steps += [ordered]@{ action='install-verify'; cli_version=$cliVersion; gui_version=$guiVersion; doctor=$doctor; language_checks=@('en-US','zh-CN','zh-TW'); path_entry=$InstallDir }
    if (-not $SkipGuiColdStart) {
        $sw = [Diagnostics.Stopwatch]::StartNew()
        $gui = Start-Process -FilePath (Join-Path $InstallDir 'clens-gui.exe') -PassThru
        Start-Sleep -Milliseconds 100
        $sw.Stop()
        $coldMs = $sw.Elapsed.TotalMilliseconds
        if (-not $gui.HasExited) { $gui.CloseMainWindow() | Out-Null; Start-Sleep -Milliseconds 100; if (-not $gui.HasExited) { $gui.Kill() } }
        Assert-That ($coldMs -lt 600) "GUI process cold-start gate exceeded 600ms: $coldMs"
        $report.steps += [ordered]@{ action='gui-cold-start'; milliseconds=[math]::Round($coldMs,2) }
    }
    $uninstall = Join-Path $InstallDir 'uninstall.exe'
    $uargs = @('/S','/NORESTART',"/LOG=$log")
    $up = Start-Process -FilePath $uninstall -ArgumentList $uargs -Wait -PassThru -WindowStyle Hidden
    Assert-That ($up.ExitCode -in @(0,3010)) "Uninstaller exit code $($up.ExitCode)"
    $registryEmpty = -not (Test-Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\CausativeLens') -and -not (Test-Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\App Paths\clens.exe')
    $associationEmpty = -not (Test-Path 'HKCU:\Software\Classes\.cllens') -and -not (Test-Path 'HKCU:\Software\Classes\CausativeLens.Session')
    $shortcutEmpty = -not (Test-Path (Join-Path ([Environment]::GetFolderPath('Desktop')) 'CausativeLens.lnk')) -and -not (Test-Path (Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\CausativeLens'))
    $userPath = (Get-ItemProperty -Path 'HKCU:\Environment' -Name Path -ErrorAction SilentlyContinue).Path
    $pathEmpty = (($userPath -split ';') -notcontains $InstallDir)
    $filesEmpty = -not (Test-Path -LiteralPath $InstallDir)
    Assert-That $filesEmpty "Install files remain at $InstallDir"
    Assert-That $registryEmpty 'Registry residual check failed'
    Assert-That $pathEmpty 'PATH residual check failed'
    Assert-That $associationEmpty 'File association residual check failed'
    Assert-That $shortcutEmpty 'Shortcut residual check failed'
    $commandAfterUninstall = Get-Command clens.exe -ErrorAction SilentlyContinue
    $doctorAfterUninstall = $null -eq $commandAfterUninstall
    Assert-That $doctorAfterUninstall 'clens remains discoverable after uninstall'
    $report.steps += [ordered]@{ action='uninstall-verify'; files=$filesEmpty; registry=$registryEmpty; path=$pathEmpty; association=$associationEmpty; shortcuts=$shortcutEmpty; doctor_after_uninstall='not-installed (command absent)' }
    Invoke-Installer $installArgs
    Assert-That (Test-Path (Join-Path $InstallDir 'clens-gui.exe')) 'Reinstall failed'
    $report.steps += [ordered]@{ action='reinstall-verify'; status='ok' }
    if (-not $KeepInstall) { Start-Process -FilePath (Join-Path $InstallDir 'uninstall.exe') -ArgumentList @('/S','/NORESTART') -Wait -WindowStyle Hidden | Out-Null }
    $report.status='pass'
    $report.finished_utc=[DateTime]::UtcNow.ToString('o')
    $out=Join-Path (Split-Path $InstallerPath) 'installer-e2e-report.json'
    $report | ConvertTo-Json -Depth 12 | Set-Content -Encoding utf8 $out
    Write-Host "Installer E2E passed. Report: $out"
} catch {
    $report.status='fail'; $report.error=$_.Exception.Message; $report.finished_utc=[DateTime]::UtcNow.ToString('o')
    $out=Join-Path (Split-Path $InstallerPath) 'installer-e2e-report.json'
    $report | ConvertTo-Json -Depth 12 | Set-Content -Encoding utf8 $out
    throw
}






