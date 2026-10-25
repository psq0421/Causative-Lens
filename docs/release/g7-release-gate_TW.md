# G7 發行閘門

只有具備以下證據才允許正式發行：x64/ARM64 安裝包 `CausativeLens-setup-x64.exe` 與 `CausativeLens-setup-arm64.exe`；每個可執行檔都有 Azure Trusted Signing 驗證與時間戳；SPDX SBOM、鎖定依賴、確定性建置中繼資料和雜湊；乾淨 Windows VM 通過靜默安裝、重新啟動、`clens doctor`、最小工作階段、GUI 冷啟動/RSS/幀率、解除安裝五查和重裝；英文、簡體中文、繁體中文文件連結全部可用。

沒有簽名憑據的機器可以建置和測試 `-dev` 包，但不能建立正式 Release。閘門刻意保持枯燥：軟體用一個可觀察的檢查接一個可觀察的檢查贏得信任。

語言：[English](g7-release-gate.md) · [简体中文](g7-release-gate_CN.md) · [繁體中文](g7-release-gate_TW.md)。
