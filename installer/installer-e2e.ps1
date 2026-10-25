[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$InstallerPath,
    [string]$Version = '0.1.1',
    [string]$InstallDir = (Join-Path $env:TEMP ('CausativeLens E2E ' + [char]0x4e2d + [char]0x6587 + ' ' + [Guid]::NewGuid().ToString('N'))),
    [switch]$SkipGuiColdStart
)
$ErrorActionPreference = 'Stop'
$InstallerPath = (Resolve-Path -LiteralPath $InstallerPath).Path
$InstallDir = [IO.Path]::GetFullPath($InstallDir)
if (Test-Path -LiteralPath $InstallDir) { throw 'E2E requires a new destination; existing directories are never deleted.' }
if (Get-Process clens* -ErrorAction SilentlyContinue) { throw 'Close CausativeLens before installer E2E.' }
$productKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\CausativeLens'
$appKeys = @('HKCU:\Software\Microsoft\Windows\CurrentVersion\App Paths\clens.exe','HKCU:\Software\Microsoft\Windows\CurrentVersion\App Paths\clens-gui.exe')
$classKeys = @('HKCU:\Software\Classes\.cllens','HKCU:\Software\Classes\CausativeLens.Session')
foreach ($key in @($productKey) + $appKeys + $classKeys) { if (Test-Path -LiteralPath $key) { throw "Existing registration: use a clean VM ($key)" } }
foreach ($view in @([Microsoft.Win32.RegistryView]::Registry64,[Microsoft.Win32.RegistryView]::Registry32)) {
    $base = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::CurrentUser,$view)
    $key = $base.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Uninstall\CausativeLens')
    try { if ($key) { throw 'An installation is registered; use a clean VM.' } } finally { if ($key) { $key.Dispose() }; $base.Dispose() }
}
$menu = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\CausativeLens'
$shortcutPaths = @((Join-Path ([Environment]::GetFolderPath('Desktop')) 'CausativeLens.lnk'),(Join-Path $menu 'CausativeLens GUI.lnk'),(Join-Path $menu 'CausativeLens CLI.lnk'))
$shortcutSnapshot = @{}
foreach ($file in $shortcutPaths) { if (Test-Path -LiteralPath $file) { $shortcutSnapshot[$file] = [IO.File]::ReadAllBytes($file) } }
$processPathBefore = $env:Path
$environmentKey = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment',$true)
$pathBefore = $environmentKey.GetValue('Path',$null,[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
$pathKindBefore = if ($null -ne $pathBefore) { $environmentKey.GetValueKind('Path') } else { $null }
$environmentKey.Dispose()
$log = Join-Path $env:TEMP ('causativelens-e2e-' + [Guid]::NewGuid().ToString('N') + '.log')
$report = [ordered]@{ schema='causativelens.installer-e2e/2'; installer=$InstallerPath; version=$Version; install_dir=$InstallDir; started_utc=[DateTime]::UtcNow.ToString('o'); steps=@(); reboot_test='not performed'; gui_performance='not measured; native window smoke only' }
function Assert-That([bool]$condition, [string]$message) { if (-not $condition) { throw $message } }
function Invoke-Installer {
    $arguments = @('/S','/NORESTART',"/LOG=$log", "/D=$InstallDir")
    $process = Start-Process -FilePath $InstallerPath -ArgumentList $arguments -Wait -PassThru -WindowStyle Hidden
    Assert-That ($process.ExitCode -in @(0,3010)) "Installer failed: $($process.ExitCode)"
}
function Invoke-Uninstaller {
    $uninstall = Join-Path $InstallDir 'uninstall.exe'
    if (Test-Path -LiteralPath $uninstall) {
        $process = Start-Process -FilePath $uninstall -ArgumentList @('/S','/NORESTART',"/LOG=$log") -Wait -PassThru -WindowStyle Hidden
        Assert-That ($process.ExitCode -in @(0,3010)) "Uninstaller failed: $($process.ExitCode)"
        $deadline = [DateTime]::UtcNow.AddSeconds(15)
        while ((Test-Path -LiteralPath $InstallDir) -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 100 }
    }
}
try {
    Invoke-Installer
    foreach ($name in @('clens.exe','clens-gui.exe','clens-daemon.exe','clens-capture.exe','uninstall.exe')) { Assert-That (Test-Path -LiteralPath (Join-Path $InstallDir $name)) "Missing payload: $name" }
    $cliVersion = (& (Join-Path $InstallDir 'clens.exe') --version | Out-String).Trim()
    $guiVersion = (& (Join-Path $InstallDir 'clens-gui.exe') --version | Out-String).Trim()
    Assert-That ($cliVersion.Contains($Version) -and $guiVersion -eq $Version) 'CLI/GUI version mismatch'
    $doctor = (& (Join-Path $InstallDir 'clens.exe') doctor --json | Out-String).Trim()
    Assert-That ($LASTEXITCODE -eq 0 -and $doctor.Contains('observe')) 'Doctor smoke failed'
    foreach ($key in @($productKey) + $appKeys + $classKeys) { Assert-That (Test-Path -LiteralPath $key) "Missing registration: $key" }
    Assert-That ((Get-ItemProperty -LiteralPath $productKey).DisplayVersion -eq $Version) 'Registry version mismatch'
    foreach ($file in $shortcutPaths) { Assert-That (Test-Path -LiteralPath $file) "Missing shortcut: $file" }
    $shortcutResult = & (Join-Path $PSScriptRoot 'test-cli-shortcut.ps1') -InstallDir $InstallDir
    $report.steps += [ordered]@{ action='install'; cli=$cliVersion; gui=$guiVersion; doctor=$doctor; cli_shortcut=$shortcutResult }
    Invoke-Installer
    $userPath = (Get-ItemProperty 'HKCU:\Environment' -Name Path).Path
    Assert-That ((@($userPath -split ';' | Where-Object { $_ -eq $InstallDir })).Count -eq 1) 'Repeated installation duplicated PATH'
    $report.steps += [ordered]@{ action='overwrite'; path_occurrences=1 }
    if (-not $SkipGuiColdStart) {
        $watch = [Diagnostics.Stopwatch]::StartNew()
        $gui = Start-Process -FilePath (Join-Path $InstallDir 'clens-gui.exe') -PassThru -WindowStyle Hidden
        try {
            $deadline = [DateTime]::UtcNow.AddSeconds(15)
            do { Start-Sleep -Milliseconds 50; $gui.Refresh() } while (-not $gui.HasExited -and $gui.MainWindowHandle -eq 0 -and [DateTime]::UtcNow -lt $deadline)
            Assert-That (-not $gui.HasExited -and $gui.MainWindowHandle -ne 0) 'GUI did not create a native window'
            $report.steps += [ordered]@{ action='gui-native-window-smoke'; first_window_ms=[math]::Round($watch.Elapsed.TotalMilliseconds,2); cold_start_budget='not assessed by this test' }
        } finally {
            if (-not $gui.HasExited) { $gui.CloseMainWindow() | Out-Null; if (-not $gui.WaitForExit(3000)) { $gui.Kill() } }
        }
    }
    Invoke-Uninstaller
    $checks = [ordered]@{
        files = -not (Test-Path -LiteralPath $InstallDir)
        registry = (@(@($productKey) + $appKeys | Where-Object { Test-Path -LiteralPath $_ })).Count -eq 0
        path = ((Get-ItemProperty 'HKCU:\Environment' -Name Path -ErrorAction SilentlyContinue).Path -split ';') -notcontains $InstallDir
        association = (@($classKeys | Where-Object { Test-Path -LiteralPath $_ })).Count -eq 0
        shortcuts = (@($shortcutPaths | Where-Object { Test-Path -LiteralPath $_ })).Count -eq 0
    }
    foreach ($check in $checks.Keys) { Assert-That $checks[$check] "Uninstall residual: $check" }
    $report.steps += [ordered]@{ action='uninstall-five-checks'; checks=$checks }
    Invoke-Installer
    Assert-That (Test-Path -LiteralPath (Join-Path $InstallDir 'clens.exe')) 'Reinstall failed'
    $report.steps += [ordered]@{ action='reinstall'; status='pass' }
    Invoke-Uninstaller
    Assert-That (-not (Test-Path -LiteralPath $InstallDir)) 'Reinstall cleanup failed'
    $report.status='pass'
} catch {
    $report.status='fail'; $report.error=$_.Exception.Message
    throw
} finally {
    try { Invoke-Uninstaller } catch { Write-Warning $_.Exception.Message }
    $environmentKey = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment',$true)
    try { if ($null -eq $pathBefore) { $environmentKey.DeleteValue('Path',$false) } else { $environmentKey.SetValue('Path',$pathBefore,$pathKindBefore) } } finally { $environmentKey.Dispose() }
    $env:Path = $processPathBefore
    foreach ($file in $shortcutPaths) { if ($shortcutSnapshot.ContainsKey($file)) { New-Item -ItemType Directory -Force -Path (Split-Path $file) | Out-Null; [IO.File]::WriteAllBytes($file,$shortcutSnapshot[$file]) } }
    $report.finished_utc=[DateTime]::UtcNow.ToString('o')
    $out=Join-Path (Split-Path $InstallerPath) 'installer-e2e-report.json'
    $report | ConvertTo-Json -Depth 12 | Set-Content -Encoding utf8 $out
    Write-Host "Installer E2E $($report.status). Report: $out"
}
