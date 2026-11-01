# CausativeLens v0.2.5 安裝器與發佈線

NSIS 安裝器按目前使用者安裝，一次安裝 `clens.exe`（CLI）、`clens-gui.exe`（GUI）、`clens-daemon.exe` 與 `clens-capture.exe`。它不寫 HKLM、不安裝服務、不載入驅動，也不向其他程序注入。只有 `clens-capture.exe` 帶有 `requireAdministrator` manifest，用於真實 ETW 擷取。

## 本機 x64 建置

```powershell
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.2.5
powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.2.5
```

沒有 Azure Trusted Signing 時，輸出為 `dist/CausativeLens-setup-x64-dev.exe`。正式命名的資產必須先通過 Trusted Signing 驗證。

## 安裝器行為

- `/S` 靜默安裝；`/D=<絕對路徑>` 自訂目錄；`/NORESTART` 禁止自動重啟；`/LOG=<path>` 寫入安裝日誌。
- 冪等寫入使用者級 PATH、HKCU Uninstall/App Paths、`.cllens` Classes、開始功能表、桌面捷徑與 GUI 預設入口。
- 檢查 WebView2；缺少時引導官方 Evergreen bootstrapper。
- 完成頁解釋 PATH、登錄與 WebView2 註冊為何可能需要重啟。
- 解除安裝時將鎖定檔案移入 `.trash/pending`；只有使用者勾選或傳入 `/CLEANUSERDATA` 才刪除會話資料。

## E2E

```powershell
powershell -ExecutionPolicy Bypass -File installer/installer-e2e.ps1 `
  -InstallerPath dist/CausativeLens-setup-x64-dev.exe
```

E2E 會在包含空格與中文的路徑中執行靜默安裝，檢查 CLI/GUI 版本、doctor、PATH、登錄、GUI 冷啟動、五項殘留，完成解除安裝與重裝。ARM64 在 ARM64 runner 上復用同一套腳本。

`v1.0.0` 之前所有版本都是 beta/測試版。完整文件見[三語目錄](../docs/README_TW.md)。
