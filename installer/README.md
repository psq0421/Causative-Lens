# CausativeLens v0.2.0 G7 瀹夎涓庡彂甯冪嚎

瀹夎鍣ㄩ〉闈㈡敮鎸?English銆佺畝浣撲腑鏂囧拰绻侀珨涓枃锛涘悓涓€绛惧悕 payload 鍙寜鏋舵瀯鍖哄垎銆傚畬鏁村弻璇?涓夎鏂囨。鍏ュ彛瑙?[docs/README.md](../docs/README.md)銆?
鏈洰褰曟槸 NSIS 涓€浣撳畨瑁呭櫒鐨勫敮涓€鍏ュ彛銆倄64 鍖呭悓鏃跺畨瑁?`clens.exe`锛圕LI锛夈€乣clens-gui.exe`锛堥粯璁?GUI锛夈€乣clens-daemon.exe` 涓?`clens-capture.exe`锛涘畨瑁呭櫒鏈韩涓哄綋鍓嶇敤鎴锋潈闄愯繍琛岋紝涓嶅啓 HKLM銆佷笉瑁呮湇鍔°€傚彧鏈?`clens-capture.exe` 鎼哄甫 `requireAdministrator` manifest銆?
## 鏈満 x64 鏋勫缓

```powershell
# 鏋勫缓 release 浜岃繘鍒躲€佹牎楠?CLI/GUI 鍚岀増鏈苟鐢熸垚 payload-manifest.json
powershell -ExecutionPolicy Bypass -File installer/build-payload.ps1 -Arch x64 -Version 0.2.0

# 缂栬瘧 NSIS锛涙棤 Trusted Signing 鏃跺彧鐢熸垚 -dev 鍖?powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.2.0
```

杈撳嚭涓?`dist/CausativeLens-setup-x64-dev.exe`銆傛寮?Release 鏂囦欢鍚嶅彧鑳芥槸 `CausativeLens-setup-x64.exe`锛屽繀椤诲湪绛惧悕楠岃瘉閫氳繃鍚庢瀯寤猴細

```powershell
$env:AZURE_TRUSTED_SIGNING_DLIB = 'C:\path\Azure.CodeSigning.Dlib.dll'
$env:AZURE_TRUSTED_SIGNING_METADATA = 'C:\path\metadata.json'
powershell -ExecutionPolicy Bypass -File installer/sign-release.ps1 -Files installer/payload/clens.exe,installer/payload/clens-gui.exe,installer/payload/clens-daemon.exe,installer/payload/clens-capture.exe -OutputStatus dist/payload-signature-status.json
# 鍙湁 payload-signature-status.json 涓?Trusted Signing 鏃舵墠鑳界敓鎴愭寮忔枃浠?powershell -ExecutionPolicy Bypass -File installer/build-installer.ps1 -Arch x64 -Version 0.2.0 -Release -Signed
powershell -ExecutionPolicy Bypass -File installer/sign-release.ps1 -Files dist/CausativeLens-setup-x64.exe
```

涔熷彲浣跨敤 `installer/build-release.ps1 -Arch x64 -Version 0.2.0` 涓€娆℃墽琛?payload 绛惧悕銆佹寮忓寘缂栬瘧銆佸畨瑁呭櫒绛惧悕鍜?E2E锛涗换涓€姝ユ病鏈?Trusted Signing 閮戒細闃绘柇銆?

`sign-release.ps1` 浣跨敤 Azure Trusted Signing 鐨?`signtool /dlib /dmdf` 娴佺▼骞惰繍琛?`signtool verify /pa /all`銆傜己灏戦厤缃椂鍙敓鎴?`signature-status.json` 鐨?`unsigned-dev` 鐘舵€侊紝姝ｅ紡鏋勫缓浼氶樆鏂紱绂佹鐢ㄨ嚜绛惧悕璇佷功浠ｆ浛鍏紑鍙戝竷绛惧悕銆侫zure 鐨勭鎴枫€佸鎴风鍜?secret 鐢?Trusted Signing DLib 鐨勫彈鏀寔鍑嵁閾炬彁渚涳紝涓嶅啓鍏ヤ粨搴撱€?
`generate-sbom.ps1` 浣跨敤閿佸畾鐨?`cargo metadata` 鐢熸垚 SPDX 2.3 鏂囦欢銆傚寘浣撲笂闄愮敱 `build-installer.ps1` 闃绘柇锛坄<25 MB`锛夈€?
## 瀹夎鍣ㄨ涓?
- `/S` 闈欓粯锛沗/D=<缁濆璺緞>` 鑷畾涔夌洰褰曪紙鏈€鍚庝竴涓弬鏁帮級锛沗/NORESTART` 绂佹鑷姩閲嶅惎锛沗/LOG=<path>` 璁板綍姝ラ锛涘嵏杞芥敮鎸?`/CLEANUSERDATA`銆?- 瀹夎鍐欏叆鐢ㄦ埛绾?PATH锛堟寜鍒嗗彿 token 骞傜瓑鍘婚噸锛夈€丠KCU Uninstall/App Paths銆乣.cllens` Classes锛屽缓绔嬫闈㈠拰寮€濮嬭彍鍗?GUI/CLI 蹇嵎鏂瑰紡銆?- WebView2 Evergreen 宸插畨瑁呭垯缁х画锛涚己澶辨椂浼樺厛鎵ц闅忓寘 bootstrapper锛屽惁鍒欏湪浜や簰瀹夎鎵撳紑寰蒋瀹樻柟寮曞椤碉紝闈欓粯瀹夎鍙褰曞彲淇鐘舵€併€?- 澶辫触鎴栭攣瀹氭枃浠朵笉浼氳鐩栧叾浠栫敤鎴锋枃浠讹紱鍗歌浇浼氬皾璇曞皢閿佸畾鏂囦欢绉诲埌 `$INSTDIR/.trash/pending`銆備細璇濇暟鎹彧鍦ㄧ敤鎴峰嬀閫夋垨浼?`/CLEANUSERDATA` 鏃舵竻鐞嗐€?- 瀹屾垚椤佃В閲?PATH銆佹敞鍐岃〃鍜?WebView2 鐢熸晥闇€瑕侀噸鍚紝骞舵彁渚涚珛鍗抽噸鍚?绋嶅悗閫夐」銆?
## x64 瀹夎鍣?E2E

```powershell
powershell -ExecutionPolicy Bypass -File installer/installer-e2e.ps1 `
  -InstallerPath dist/CausativeLens-setup-x64-dev.exe
```

鑴氭湰浼氬湪鍖呭惈绌烘牸鍜屼腑鏂囩殑涓存椂鐩綍鎵ц闈欓粯瀹夎銆丆LI/GUI 鐗堟湰妫€鏌ャ€乣clens doctor --json`銆丳ATH/娉ㄥ唽琛?鍏宠仈/蹇嵎鏂瑰紡妫€鏌ャ€丟UI 鍐峰惎鍔紙鐩爣 `<600 ms`锛夈€佸嵏杞戒簲椤规畫鐣欐鏌ヤ笌閲嶈锛屽苟鎶婅瘉鎹啓鍒?`dist/installer-e2e-report.json`銆侫RM64 浣跨敤鍚屼竴鑴氭湰鍜?NSIS 鑴氭湰锛屽湪 ARM64 runner 涓婁紶鍏?`-Arch arm64`銆?





