; CausativeLens v0.1.0 G7 per-user installer.
; This script uses HKCU only. It installs one GUI/CLI payload and keeps
; clens-capture.exe as the only requireAdministrator binary.
Unicode true
ManifestSupportedOS win10
RequestExecutionLevel user
SetCompressor /SOLID lzma
SetDatablockOptimize on
SetOverwrite ifnewer
BrandingText "CausativeLens"
InstallDir "$LOCALAPPDATA\causativelens"

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"
!include "WinMessages.nsh"
!include "WordFunc.nsh"
!include "StrFunc.nsh"
${Using:StrFunc} StrStr
${Using:StrFunc} UnStrStr
!define SMTO_ABORTIFHUNG 0x0002
!insertmacro GetParameters
!insertmacro GetOptions
!insertmacro WordFind
!insertmacro WordReplace
!insertmacro VersionCompare

!ifndef PRODUCT_VERSION
  !define PRODUCT_VERSION "0.1.0"
!endif
!ifndef ARCH
  !define ARCH "x64"
!endif
!ifndef PAYLOAD_DIR
  !define PAYLOAD_DIR "payload"
!endif
!ifndef OUT_FILE
  !define OUT_FILE "CausativeLens-setup-${ARCH}-dev.exe"
!endif

; The payload is identical for all locales; only the installer conversation is
; localized so one signed binary can welcome three language communities.
LangString WELCOME_TITLE 1033 "CausativeLens installation"
LangString WELCOME_TITLE 2052 "CausativeLens 安装"
LangString WELCOME_TITLE 1028 "CausativeLens 安裝"
LangString WELCOME_TEXT 1033 "One installation provides the GUI and CLI. The GUI is the default entry point; the CLI is available in your terminal. Installation is per-user and does not require administrator permission."
LangString WELCOME_TEXT 2052 "一次安装同时提供 GUI 与 CLI。GUI 是默认入口，CLI 可在终端使用。安装为当前用户执行，不需要管理员权限。"
LangString WELCOME_TEXT 1028 "一次安裝同時提供 GUI 與 CLI。GUI 是預設入口，CLI 可在終端機使用。安裝為目前使用者執行，不需要系統管理員權限。"
LangString FINISH_TITLE 1033 "CausativeLens is installed"
LangString FINISH_TITLE 2052 "CausativeLens 已安装"
LangString FINISH_TITLE 1028 "CausativeLens 已安裝"
LangString FINISH_TEXT 1033 "GUI, CLI, daemon and capture are installed. Restarting lets PATH, file associations and WebView2 registration reach new processes."
LangString FINISH_TEXT 2052 "GUI、CLI、daemon 和 capture 已安装。PATH、文件关联与 WebView2 注册可能需要重启后才对新进程生效。"
LangString FINISH_TEXT 1028 "GUI、CLI、daemon 與 capture 已安裝。PATH、檔案關聯與 WebView2 註冊可能需要重新啟動後才會對新程序生效。"
LangString WEBVIEW_MISSING 1033 "WebView2 Evergreen is not installed. Open Microsoft's official installer page?"
LangString WEBVIEW_MISSING 2052 "系统未检测到 WebView2 Evergreen。是否打开微软官方安装页面？"
LangString WEBVIEW_MISSING 1028 "系統未偵測到 WebView2 Evergreen。是否開啟 Microsoft 官方安裝頁面？"
LangString RESTART_TEXT 1033 "Installation is complete. PATH, file associations and WebView2 registration may need a restart. Restart now?$\r$\n$\r$\nChoose No to restart later."
LangString RESTART_TEXT 2052 "安装已完成。PATH、文件关联和 WebView2 注册建议重启后生效。立即重启？$\r$\n$\r$\n选择“否”可稍后手动重启。"
LangString RESTART_TEXT 1028 "安裝已完成。PATH、檔案關聯和 WebView2 註冊建議重新啟動後生效。立即重新啟動？$\r$\n$\r$\n選擇「否」可稍後手動重新啟動。"
LangString DOWNGRADE_TEXT 1033 "Version $TempValue is already installed and is newer than ${PRODUCT_VERSION}. Downgrades are blocked by default; use /ALLOWDOWNGRADE if you really need one."
LangString DOWNGRADE_TEXT 2052 "检测到已安装版本 $TempValue，高于 ${PRODUCT_VERSION}。默认禁止降级；如确需降级请使用 /ALLOWDOWNGRADE。"
LangString DOWNGRADE_TEXT 1028 "偵測到已安裝版本 $TempValue，高於 ${PRODUCT_VERSION}。預設禁止降級；如確需降級請使用 /ALLOWDOWNGRADE。"
LangString UNINSTALL_DATA_TEXT 1033 "Also delete session data in %LOCALAPPDATA%\\causativelens?"
LangString UNINSTALL_DATA_TEXT 2052 "是否同时删除 %LOCALAPPDATA%\\causativelens 中的会话数据？"
LangString UNINSTALL_DATA_TEXT 1028 "是否同時刪除 %LOCALAPPDATA%\\causativelens 中的工作階段資料？"

Name "CausativeLens ${PRODUCT_VERSION}"
OutFile "${OUT_FILE}"
VIProductVersion "${PRODUCT_VERSION}.0"
VIAddVersionKey /LANG=2052 "ProductName" "CausativeLens"
VIAddVersionKey /LANG=2052 "ProductVersion" "${PRODUCT_VERSION}"
VIAddVersionKey /LANG=2052 "FileVersion" "${PRODUCT_VERSION}.0"
VIAddVersionKey /LANG=2052 "FileDescription" "CausativeLens GUI and CLI installer"
VIAddVersionKey /LANG=2052 "CompanyName" "CausativeLens"
VIAddVersionKey /LANG=2052 "LegalCopyright" "Copyright (c) CausativeLens contributors"
VIAddVersionKey /LANG=1028 "ProductName" "CausativeLens"
VIAddVersionKey /LANG=1033 "ProductName" "CausativeLens"
VIAddVersionKey /LANG=1028 "ProductVersion" "${PRODUCT_VERSION}"
VIAddVersionKey /LANG=1033 "ProductVersion" "${PRODUCT_VERSION}"
VIAddVersionKey /LANG=1028 "FileVersion" "${PRODUCT_VERSION}.0"
VIAddVersionKey /LANG=1033 "FileVersion" "${PRODUCT_VERSION}.0"
VIAddVersionKey /LANG=1028 "FileDescription" "CausativeLens GUI and CLI installer"
VIAddVersionKey /LANG=1033 "FileDescription" "CausativeLens GUI and CLI installer"
VIAddVersionKey /LANG=1028 "CompanyName" "CausativeLens"
VIAddVersionKey /LANG=1033 "CompanyName" "CausativeLens"
VIAddVersionKey /LANG=1028 "LegalCopyright" "Copyright (c) CausativeLens contributors"
VIAddVersionKey /LANG=1033 "LegalCopyright" "Copyright (c) CausativeLens contributors"

!define MUI_ABORTWARNING
!define MUI_ICON "${PAYLOAD_DIR}\causativelens.ico"
!define MUI_UNICON "${PAYLOAD_DIR}\causativelens.ico"
!define MUI_WELCOMEPAGE_TITLE "$(WELCOME_TITLE)"
!define MUI_WELCOMEPAGE_TEXT "$(WELCOME_TEXT)"
!define MUI_FINISHPAGE_TITLE "$(FINISH_TITLE)"
!define MUI_FINISHPAGE_TEXT "$(FINISH_TEXT)"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "TradChinese"

Var InstallLogPath
Var NoRestart
Var CleanUserData
Var AllowDowngrade
Var RebootRequired
Var InstallerCmdLine
Var TempValue
Var TempValue2
Var TempValue3

!define PRODUCT_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\CausativeLens"
!define APP_PATHS_KEY "Software\Microsoft\Windows\CurrentVersion\App Paths"
!define CLASSES_KEY "Software\Classes"

Function WriteLog
  Exch $0
  ${If} $InstallLogPath == ""
    Pop $0
    Return
  ${EndIf}
  ClearErrors
  FileOpen $1 "$InstallLogPath" a
  ${IfNot} ${Errors}
    FileWrite $1 "$0$\r$\n"
    FileClose $1
  ${EndIf}
  Pop $0
FunctionEnd

Function ParseCommonOptions
  ${GetParameters} $InstallerCmdLine
  StrCpy $NoRestart 0
  StrCpy $CleanUserData 0
  StrCpy $AllowDowngrade 0
  StrCpy $RebootRequired 0

  ${GetOptions} "$InstallerCmdLine" "/NORESTART" $TempValue
  ${IfNot} ${Errors}
    StrCpy $NoRestart 1
  ${EndIf}
  ${GetOptions} "$InstallerCmdLine" "/CLEANUSERDATA" $TempValue
  ${IfNot} ${Errors}
    StrCpy $CleanUserData 1
  ${EndIf}
  ${GetOptions} "$InstallerCmdLine" "/ALLOWDOWNGRADE" $TempValue
  ${IfNot} ${Errors}
    StrCpy $AllowDowngrade 1
  ${EndIf}
  ${GetOptions} "$InstallerCmdLine" "/LOG=" $TempValue
  ${IfNot} ${Errors}
    StrCpy $InstallLogPath $TempValue
  ${EndIf}
  ${If} $InstallLogPath == ""
    StrCpy $InstallLogPath "$TEMP\CausativeLens-install.log"
  ${EndIf}
  CreateDirectory "$TEMP"
  Push "CausativeLens ${PRODUCT_VERSION} ${ARCH} installer started"
  Call WriteLog
FunctionEnd

Function NotifyShell
  ; Refresh Explorer's per-user PATH and .cllens association cache.
  System::Call 'user32::SendMessageTimeout(i ${HWND_BROADCAST}, i ${WM_SETTINGCHANGE}, i 0, t "Environment", i ${SMTO_ABORTIFHUNG}, i 5000, *i .s)'
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0x0000, p 0, p 0)'
FunctionEnd

Function AddUserPath
  ReadRegStr $TempValue HKCU "Environment" "Path"
  ${If} $TempValue == ""
    StrCpy $TempValue "$INSTDIR"
  ${Else}
    ClearErrors
    ${WordFind} "$TempValue" ";" "E/$INSTDIR" $TempValue2
    ${If} ${Errors}
      StrCpy $TempValue "$TempValue;$INSTDIR"
    ${EndIf}
  ${EndIf}
  WriteRegExpandStr HKCU "Environment" "Path" "$TempValue"
  StrCpy $RebootRequired 1
  Push "User PATH updated (idempotent): $INSTDIR"
  Call WriteLog
FunctionEnd

Function RemoveUserPath
  ReadRegStr $TempValue HKCU "Environment" "Path"
  ${If} $TempValue == ""
    Return
  ${EndIf}
  ${If} $TempValue == "$INSTDIR"
    StrCpy $TempValue ""
  ${Else}
    ${WordReplace} "$TempValue" ";$INSTDIR" "" "*" $TempValue2
    ${WordReplace} "$TempValue2" "$INSTDIR;" "" "*" $TempValue
  ${EndIf}
  WriteRegExpandStr HKCU "Environment" "Path" "$TempValue"
  StrCpy $RebootRequired 1
  Push "User PATH entry removed: $INSTDIR"
  Call WriteLog
FunctionEnd

Function RegisterProduct
  WriteRegStr HKCU "${PRODUCT_KEY}" "DisplayName" "CausativeLens"
  WriteRegStr HKCU "${PRODUCT_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr HKCU "${PRODUCT_KEY}" "Publisher" "CausativeLens"
  WriteRegStr HKCU "${PRODUCT_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${PRODUCT_KEY}" "InstallSource" "$EXEDIR"
  WriteRegStr HKCU "${PRODUCT_KEY}" "Architecture" "${ARCH}"
  WriteRegDWORD HKCU "${PRODUCT_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${PRODUCT_KEY}" "NoRepair" 1
  WriteRegStr HKCU "${PRODUCT_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "${PRODUCT_KEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S /NORESTART'
  WriteRegStr HKCU "${APP_PATHS_KEY}\clens.exe" "" "$INSTDIR\clens.exe"
  WriteRegStr HKCU "${APP_PATHS_KEY}\clens.exe" "Path" "$INSTDIR"
  WriteRegStr HKCU "${APP_PATHS_KEY}\clens-gui.exe" "" "$INSTDIR\clens-gui.exe"
  WriteRegStr HKCU "${APP_PATHS_KEY}\clens-gui.exe" "Path" "$INSTDIR"

  WriteRegStr HKCU "${CLASSES_KEY}\.cllens" "" "CausativeLens.Session"
  WriteRegStr HKCU "${CLASSES_KEY}\.cllens" "Content Type" "application/x-causativelens-session"
  WriteRegStr HKCU "${CLASSES_KEY}\CausativeLens.Session" "" "CausativeLens session"
  WriteRegStr HKCU "${CLASSES_KEY}\CausativeLens.Session\DefaultIcon" "" "$INSTDIR\clens-gui.exe,0"
  WriteRegStr HKCU "${CLASSES_KEY}\CausativeLens.Session\shell\open\command" "" '"$INSTDIR\clens-gui.exe" "%1"'
  Push "HKCU product, App Paths and .cllens classes registered"
  Call WriteLog
FunctionEnd

Function CreateProductShortcuts
  CreateDirectory "$SMPROGRAMS\CausativeLens"
  CreateShortCut "$DESKTOP\CausativeLens.lnk" "$INSTDIR\clens-gui.exe"
  CreateShortCut "$SMPROGRAMS\CausativeLens\CausativeLens GUI.lnk" "$INSTDIR\clens-gui.exe"
  CreateShortCut "$SMPROGRAMS\CausativeLens\CausativeLens CLI.lnk" "$INSTDIR\clens.exe"
  Push "Desktop and Start Menu GUI/CLI shortcuts created"
  Call WriteLog
FunctionEnd

Function TestWebView2
  StrCpy $TempValue ""
  SetRegView 64
  ReadRegStr $TempValue HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4A8F-8E5C-6C4C6C6A4A7A}" "pv"
  ${If} $TempValue == ""
    ReadRegStr $TempValue HKLM "SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4A8F-8E5C-6C4C6C6A4A7A}" "pv"
  ${EndIf}
  ${If} $TempValue == ""
    ReadRegStr $TempValue HKCU "Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4A8F-8E5C-6C4C6C6A4A7A}" "pv"
  ${EndIf}
  SetRegView 32
  ${If} $TempValue == ""
    ReadRegStr $TempValue HKLM "SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4A8F-8E5C-6C4C6C6A4A7A}" "pv"
  ${EndIf}
  SetRegView 64
  ${If} $TempValue != ""
    Push "WebView2 Evergreen detected: $TempValue"
    Call WriteLog
    Return
  ${EndIf}
  Push "WebView2 Evergreen is missing"
  Call WriteLog
  IfFileExists "$INSTDIR\MicrosoftEdgeWebview2Setup.exe" 0 webview_missing
    ExecWait '"$INSTDIR\MicrosoftEdgeWebview2Setup.exe" /silent /install' $TempValue2
    ${If} $TempValue2 == "0"
      StrCpy $RebootRequired 1
      Push "WebView2 bootstrapper completed"
      Call WriteLog
      Return
    ${EndIf}
  webview_missing:
  IfSilent webview_silent
    MessageBox MB_ICONEXCLAMATION|MB_YESNO $(WEBVIEW_MISSING) IDYES webview_open
    Goto webview_done
  webview_open:
    ExecShell open "https://go.microsoft.com/fwlink/p/?LinkId=2124703"
    Goto webview_done
  webview_silent:
    Push "WebView2 unavailable; GUI will show the repair link on first launch"
    Call WriteLog
  webview_done:
FunctionEnd

Function WriteInstallManifest
  ClearErrors
  FileOpen $TempValue "$INSTDIR\install-manifest.json" w
  ${IfNot} ${Errors}
    FileWrite $TempValue '{"product":"CausativeLens","version":"${PRODUCT_VERSION}","arch":"${ARCH}","gui":"clens-gui.exe","cli":"clens.exe","mode":"observe"}$\r$\n'
    FileClose $TempValue
  ${EndIf}
FunctionEnd

Function StopProductProcesses
  nsExec::ExecToLog 'taskkill.exe /F /T /IM clens-gui.exe'
  nsExec::ExecToLog 'taskkill.exe /F /T /IM clens-daemon.exe'
  nsExec::ExecToLog 'taskkill.exe /F /T /IM clens-capture.exe'
  Sleep 400
FunctionEnd

Function MoveToTrash
  StrCpy $TempValue "$INSTDIR\.trash\pending"
  CreateDirectory "$TempValue"
  ${If} ${FileExists} "$0"
    Rename "$0" "$TempValue\$1"
    ${If} ${Errors}
      Push "locked path could not move to .trash: $0"
      Call WriteLog
    ${Else}
      Push "locked path moved to .trash: $TempValue\$1"
      Call WriteLog
    ${EndIf}
  ${EndIf}
FunctionEnd

Function DeleteInstalledFile
  Pop $0
  ${GetFileName} "$0" $1
  Delete "$0"
  ${If} ${FileExists} "$0"
    Call MoveToTrash
  ${EndIf}
FunctionEnd

Function CheckExistingVersion
  ReadRegStr $TempValue HKCU "${PRODUCT_KEY}" "DisplayVersion"
  ${If} $TempValue == ""
    Return
  ${EndIf}
  ${VersionCompare} "$TempValue" "${PRODUCT_VERSION}" $TempValue2
  ${If} $TempValue2 == "1"
    ${If} $AllowDowngrade == "0"
      MessageBox MB_ICONSTOP $(DOWNGRADE_TEXT)
      Abort
    ${EndIf}
  ${EndIf}
FunctionEnd

Function .onInit
  SetShellVarContext current
  !insertmacro MUI_LANGDLL_DISPLAY
  Call ParseCommonOptions
  Call CheckExistingVersion
  Call StopProductProcesses
FunctionEnd

Function .onInstSuccess
  Call NotifyShell
  Push "Installation completed successfully"
  Call WriteLog
  ${If} $RebootRequired == "1"
    ${If} $NoRestart == "0"
      ${IfNot} ${Silent}
        MessageBox MB_YESNO|MB_ICONINFORMATION $(RESTART_TEXT) IDYES do_restart
      ${EndIf}
    ${EndIf}
  ${EndIf}
  Return
  do_restart:
    Reboot
FunctionEnd

Section "CausativeLens GUI + CLI" SEC_MAIN
  SetOutPath "$INSTDIR"
  CreateDirectory "$INSTDIR"
  File "${PAYLOAD_DIR}\clens.exe"
  File "${PAYLOAD_DIR}\clens-gui.exe"
  File "${PAYLOAD_DIR}\clens-daemon.exe"
  File "${PAYLOAD_DIR}\clens-capture.exe"
  File "${PAYLOAD_DIR}\clens-capture.exe.manifest"
  File /nonfatal "${PAYLOAD_DIR}\MicrosoftEdgeWebview2Setup.exe"
  File /nonfatal "${PAYLOAD_DIR}\payload-manifest.json"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  Call RegisterProduct
  Call AddUserPath
  Call CreateProductShortcuts
  Call TestWebView2
  Call WriteInstallManifest
SectionEnd


Function un.WriteLog
  Exch $0
  ${If} $InstallLogPath != ""
    ClearErrors
    FileOpen $1 "$InstallLogPath" a
    ${IfNot} ${Errors}
      FileWrite $1 "$0$\r$\n"
      FileClose $1
    ${EndIf}
  ${EndIf}
  Pop $0
FunctionEnd

Function un.ParseCommonOptions
  ${GetParameters} $InstallerCmdLine
  StrCpy $NoRestart 0
  StrCpy $CleanUserData 0
  ${GetOptions} "$InstallerCmdLine" "/NORESTART" $TempValue
  ${IfNot} ${Errors}
    StrCpy $NoRestart 1
  ${EndIf}
  ${GetOptions} "$InstallerCmdLine" "/CLEANUSERDATA" $TempValue
  ${IfNot} ${Errors}
    StrCpy $CleanUserData 1
  ${EndIf}
  ${GetOptions} "$InstallerCmdLine" "/LOG=" $TempValue
  ${IfNot} ${Errors}
    StrCpy $InstallLogPath $TempValue
  ${EndIf}
  ${If} $InstallLogPath == ""
    StrCpy $InstallLogPath "$TEMP\CausativeLens-uninstall.log"
  ${EndIf}
  Push "CausativeLens uninstaller started"
  Call un.WriteLog
FunctionEnd

Function un.StopProductProcesses
  nsExec::ExecToLog 'taskkill.exe /F /T /IM clens-gui.exe'
  nsExec::ExecToLog 'taskkill.exe /F /T /IM clens-daemon.exe'
  nsExec::ExecToLog 'taskkill.exe /F /T /IM clens-capture.exe'
  Sleep 400
FunctionEnd

Function un.NotifyShell
  System::Call 'user32::SendMessageTimeout(i ${HWND_BROADCAST}, i ${WM_SETTINGCHANGE}, i 0, t "Environment", i ${SMTO_ABORTIFHUNG}, i 5000, *i .s)'
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0x0000, p 0, p 0)'
FunctionEnd

Function un.RemoveProductShortcuts
  Delete "$DESKTOP\CausativeLens.lnk"
  Delete "$SMPROGRAMS\CausativeLens\CausativeLens GUI.lnk"
  Delete "$SMPROGRAMS\CausativeLens\CausativeLens CLI.lnk"
  RMDir "$SMPROGRAMS\CausativeLens"
FunctionEnd

Function un.RemoveProduct
  DeleteRegKey HKCU "${PRODUCT_KEY}"
  DeleteRegKey HKCU "${APP_PATHS_KEY}\clens.exe"
  DeleteRegKey HKCU "${APP_PATHS_KEY}\clens-gui.exe"
  DeleteRegKey HKCU "${CLASSES_KEY}\.cllens"
  DeleteRegKey HKCU "${CLASSES_KEY}\CausativeLens.Session"
FunctionEnd

Function un.RemoveUserPath
  ReadRegStr $TempValue HKCU "Environment" "Path"
  ${If} $TempValue == ""
    Return
  ${EndIf}
  StrCpy $TempValue2 ""
  StrCpy $TempValue3 "$TempValue"
  un_path_loop:
    ${If} $TempValue3 == ""
      Goto un_path_done
    ${EndIf}
    ${UnStrStr} $0 "$TempValue3" ";"
    ${If} $0 == ""
      StrCpy $1 "$TempValue3"
      StrCpy $TempValue3 ""
    ${Else}
      StrLen $2 "$TempValue3"
      StrLen $3 "$0"
      IntOp $4 $2 - $3
      StrCpy $1 "$TempValue3" $4
      IntOp $4 $4 + 1
      StrCpy $TempValue3 "$TempValue3" "" $4
    ${EndIf}
    ${If} $1 != ""
      ${If} $1 != "$INSTDIR"
        ${If} $TempValue2 == ""
          StrCpy $TempValue2 "$1"
        ${Else}
          StrCpy $TempValue2 "$TempValue2;$1"
        ${EndIf}
      ${EndIf}
    ${EndIf}
    Goto un_path_loop
  un_path_done:
  ${If} $TempValue2 == ""
    DeleteRegValue HKCU "Environment" "Path"
  ${Else}
    WriteRegExpandStr HKCU "Environment" "Path" "$TempValue2"
  ${EndIf}
FunctionEnd

Function un.MoveToTrash
  StrCpy $TempValue "$INSTDIR\.trash\pending"
  CreateDirectory "$TempValue"
  ${If} ${FileExists} "$0"
    Rename "$0" "$TempValue\$1"
  ${EndIf}
FunctionEnd

Function un.DeleteInstalledFile
  Pop $0
  ${GetFileName} "$0" $1
  Delete "$0"
  ${If} ${FileExists} "$0"
    Call un.MoveToTrash
  ${EndIf}
FunctionEnd

Function un.RemoveSessionData
  Delete "$LOCALAPPDATA\causativelens\*.cllens"
  Delete "$LOCALAPPDATA\causativelens\*.clspool"
  RMDir /r "$LOCALAPPDATA\causativelens\sessions"
  RMDir /r "$LOCALAPPDATA\causativelens\spool"
  RMDir /r "$LOCALAPPDATA\causativelens\data"
FunctionEnd

Function un.Main
  Call un.ParseCommonOptions
  Call un.StopProductProcesses
  Call un.RemoveProductShortcuts
  Call un.RemoveProduct
  Call un.RemoveUserPath
  Push "$INSTDIR\clens.exe"
  Call un.DeleteInstalledFile
  Push "$INSTDIR\clens-gui.exe"
  Call un.DeleteInstalledFile
  Push "$INSTDIR\clens-daemon.exe"
  Call un.DeleteInstalledFile
  Push "$INSTDIR\clens-capture.exe"
  Call un.DeleteInstalledFile
  Push "$INSTDIR\clens-capture.exe.manifest"
  Call un.DeleteInstalledFile
  Delete "$INSTDIR\install-manifest.json"
  Delete "$INSTDIR\payload-manifest.json"
  ${If} $CleanUserData == "1"
    Call un.RemoveSessionData
  ${EndIf}
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  Call un.NotifyShell
  Push "CausativeLens uninstall completed; five residual checks are delegated to installer-e2e.ps1"
  Call un.WriteLog
FunctionEnd

Function un.onInit
  SetShellVarContext current
  Call un.ParseCommonOptions
  ${IfNot} ${Silent}
    MessageBox MB_YESNO|MB_ICONQUESTION $(UNINSTALL_DATA_TEXT) IDYES un_clean_data
    Goto un_options_done
  un_clean_data:
    StrCpy $CleanUserData 1
  un_options_done:
  ${EndIf}
FunctionEnd

Section "Uninstall"
  Call un.Main
SectionEnd









