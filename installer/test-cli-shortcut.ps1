[CmdletBinding()]
param(
    [string]$ShortcutPath = (Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\CausativeLens\CausativeLens CLI.lnk'),
    [Parameter(Mandatory=$true)][string]$InstallDir
)
$ErrorActionPreference = 'Stop'
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut((Resolve-Path -LiteralPath $ShortcutPath).Path)
if ((Split-Path -Leaf $shortcut.TargetPath) -ne 'cmd.exe') { throw 'CLI shortcut must open a persistent terminal.' }
if ($shortcut.Arguments -notmatch '^/D /K ' -or -not $shortcut.Arguments.Contains(('"' + (Join-Path $InstallDir 'clens.exe') + '" --help'))) { throw 'Shortcut arguments must quote the installed CLI and preserve the prompt.' }
if ($shortcut.WorkingDirectory.TrimEnd('\') -ne $InstallDir.TrimEnd('\')) { throw 'Shortcut working directory differs from the installation directory.' }
$process = Start-Process -FilePath $shortcut.TargetPath -ArgumentList $shortcut.Arguments -WorkingDirectory $shortcut.WorkingDirectory -PassThru -WindowStyle Hidden
try {
    Start-Sleep -Milliseconds 1200
    if ($process.HasExited) { throw "CLI terminal closed immediately (exit $($process.ExitCode))." }
    [pscustomobject]@{ status='pass'; target=$shortcut.TargetPath; arguments=$shortcut.Arguments; working_directory=$shortcut.WorkingDirectory; remained_open_ms=1200 }
} finally {
    if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force }
}
