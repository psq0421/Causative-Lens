# 开放问题、冲突与默认建议

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

| 编号 | 来源/冲突 | 影响 | 默认建议 |
|---|---|---|---|
| OQ-001 | 蓝图写 M0–M5/v0.x；本请求要求 M0–M7/v0.1.0 | 版本与闸门 | 采用本请求：G0–G7同一 v0.1.0列车；蓝图表18判据改名引用 |
| OQ-002 | 蓝图第4.3称 CLI 优先、GUI后置；本请求要求 v0.1.0 GUI | 交付顺序 | GUI在G5实现，底层仍共享 daemon；CLI/GUI是同workspace前端crate |
| OQ-003 | 蓝图把特权主进程描述为 `clens`；R4要求仅 capture UAC | 权限模型 | daemon/CLI/GUI标准用户，只有 `clens-capture` requireAdministrator |
| OQ-004 | 蓝图安装包目标<15MB；R7要求<25MB | 性能合同 | v0.1.0采用<25MB硬门禁，旧指标作为优化目标记录 |
| OQ-005 | 蓝图未规定单一 Release 产物；R1规定仅两个 exe | 发布 | Release仅 `CausativeLens-setup-x64.exe` 与 `...-arm64.exe`，zip仅CI |
| OQ-006 | 蓝图保留 CausativeLens 大小写路径；用户要求 `%LOCALAPPDATA%\causativelens` | 兼容 | 新安装只用小写目录；迁移旧目录列入 v0.2.0兼容任务 |
| OQ-007 | 命名是否连 crate 一并改名 | 生态与工作量 | v0.1.0保留 `clens-*`；彻底改名需单独 ADR |
| OQ-008 | G 闸门周期是否压缩 | 计划可信度 | 默认按表中 21 周单人全职+并行 CI；若单人节奏不足，优先保留出口判据而延长周期 |
| OQ-009 | 单人节奏假设是否成立 | 资源风险 | 默认按一名主工程师、Codex辅助、可用 x64/ARM64 CI；若无ARM实机，ARM冒烟不阻塞但必须有runner证据 |
| OQ-010 | 干预层延后后的文档口径 | 用户预期 | 所有 v0.1.0文档标明 observe-only；schema保留表但无写路径；第12.4/19/21仅引用为v0.2.0输入 |
| OQ-011 | WebView2离线安装包是否随发版 | 包体与网络 | 默认在线 Evergreen+离线bootstrapper引导，不把Runtime打进主安装包 |
| OQ-012 | Azure Trusted Signing账号/区域可用性 | 发布阻断 | 默认无凭据则阻断G7 Release，不发布未签名正式包 |
| OQ-013 | 
| OQ-013 | 提取到的 DOCX 目录显示第1–18章，用户要求按21章通读 | 章节编号与蓝图版本可能不一致 | 以正文已有章节和用户点名的第19/21章范围作为输入；待人类提供缺失章节原文时补充交叉引用，不改变 v0.1.0 闸门判据 |
| OQ-014 | 新要求称 v0.x 纯 CLI、v1 才有 GUI；已批准方案要求 v0.1.0 G6 GUI | 版本范围冲突 | 默认保留已批准的 v0.1.0 GUI；“CLI 优先”解释为核心路径和命令语义优先，不删除 G6 |
| OQ-015 | 新要求包体 `<15MB`、RSS `<120MB`；R7 已批准 `<25MB`、`<150MB` | 性能红线冲突 | 默认采用 R7 作为 v0.1.0 硬门；更严格数字作为 stretch target，需人类改红线后升级 |
| OQ-016 | 新要求 crossbeam/ArcSwap、windows-rs/bytemuck、rusqlite/zstd；当前代码尚未引入 | 依赖与实现缺口 | 先完成依赖审计和线程/存储契约；按 G1/G3/G4 分卡引入，未完成前不宣称热路径或 SQLite writer 已实现 |

| OQ-017 | 蓝图规定 `SysEvent.ts_qpc` 为 100 ns QPC 坐标，但现有对齐接口仍以毫秒参数直接比较 | 直接比较会把 QPC uptime 与 Harness UTC 混为一个坐标，破坏时间评分 | 默认在 G3 关闭：由 capture `ClockAnchor` 统一换算为 UTC 100 ns/毫秒，再允许对齐；在此之前仅把值当 fixture 坐标，不宣称真实跨时钟精度 |
| OQ-018 | 蓝图要求 session ID 为“启动 UTC + 随机后缀”并作为 `.cllens` 主键，现有 RPC ticket 使用 UUID | 直接替换会破坏 Named Pipe/RPC 客户端和旧会话恢复 | 默认增加独立 `session_key`/文件 stem，RPC UUID 作为兼容票据；G7 前决定是否把文件名切换为 `YYYYMMDDTHHMMSSZ-<suffix>.cllens` |
| OQ-019 | 蓝图 `TargetPattern::Path` 写作 `globset::GlobMatcher`，但跨模块 IR 要 serde/plain data | matcher 对象不可稳定序列化且携带实现状态 | 默认 IR 只存规范化 glob 字符串，由 `clens-core` 编译 matcher；契约测试锁定字符串规范化规则 |
| OQ-020 | `is_rundown`、AssertionPool 矛盾状态属于当前实现扩展，蓝图基础结构未列出 | 删除会破坏现有 spool/诊断语义，保留会扩大 IR | 默认保留 `is_rundown` 作为 serde 默认兼容字段；矛盾状态留在池索引，不进入 Assertion；若需要跨进程传递再升 schema |
| OQ-021 | Windows provider manifest 的 keyword 数值 mask 不能从博客或记忆确定，当前代码仍保留兼容全 mask | 错误 mask 会静默丢失事件或引入高频噪声 | 默认由真实 Windows runner 查询 manifest 后生成并锁定 `keyword_mask`，在此之前禁止宣称开销达标 |
| OQ-022 | 真实 `EVENT_RECORD`/TDH oracle 需要 Windows SDK、提权 runner 和稳定 ETL fixture | 本机纯 Rust fixture 只能验证字段逻辑，不能证明布局正确 | 默认 G1 增加 Windows runner 门禁；TDH 失败时阻断手工 decoder 发布 |
| OQ-023 | capture 管理员控制面如何把 spool 写权限安全下放给标准用户，以及 runas 拒绝如何回传 | ACL 过宽会扩大权限面，过窄会造成静默写失败 | 默认使用专用 spool 子目录 ACL + CL-1001 错误协议，不授予安装目录或控制管道写权限 |
| OQ-024 | FileObject 28 分片、262144 上限、TTL/LRU 的具体淘汰顺序尚未接线 | 过早淘汰会使 FileRead/FileWrite 无法归因，永不淘汰会造成内存增长 | 默认先实现容量/TTL 双阈值和可见 eviction 指标，再进行性能调参 |
| OQ-025 | ring 80% 扩批、95% 丢弃低价值 FileRead 的运行时策略尚未实现 | 没有固定策略就无法解释 buffer loss 与 Unobserved 的关系 | 默认丢弃只允许发生在明确的低价值类别，并写入 drop 计数和 manifest；不得动态静默丢弃 |
| OQ-026 | 第 8.1 要求 `ReadDirectoryChangesW` 目录通知，但日志轮换、重命名和 overlapped 取消的生命周期尚未由 daemon 统一管理 | 可能在轮换竞态下漏唤醒或阻塞关闭 | 默认先用同步目录通知 + 文件 offset/截断检测；G5 Windows runner 覆盖轮换和取消后再切 overlapped || OQ-027 | 第 8.2.2 指定 Qwen ONNX int4，但模型版本、许可证、运行时 DLL 与 量化校验值未冻结 | 直接接入会造成不可复现构建或误报兼容 | 默认 v0.1.0 只保留 feature-gated SPI 和 unavailable 结果；模型包、哈希、许可证在 v0.2.0 单独拍板 || OQ-028 | 用户要求 18 个运行时近三个月兼容，但当前没有每个运行时的真实 JSONL 样本和版本探针 | Tier3 fallback 不能证明字段级兼容 | 默认每个运行时至少收集一个带版本和采样日期的 fixture；缺样本时 doctor 明确标 `tier3-jsonl-unverified`，不做强制设限 || OQ-029 | `HarnessEvent.raw_json` 新增字段会改变跨模块序列化大小 | 旧 `.cllens` 回放可能缺少原始记录 | 默认使用 serde default 兼容缺失字段；schema_v 仍只在破坏性变更时升 major || OQ-030 | 蓝图要求 QueryDosDevice、8.3、符号链接和设备路径解析，但这些 API 需要 Windows capture 权限与缓存生命周期 | 在 core 中直接调用会破坏纯计算和离线回放 | 默认由 capture 生成 PathInterner snapshot，core 只接受规范化字符串表；G3 以 Windows golden fixture 验收 || OQ-031 | 断言主体 pid 依赖 run/attach 的 Harness tool→子进程锚定，当前 RPC 事件未携带完整进程上下文 | 未知主体时 process 分数无法证明为真实直接归属 | 默认保留 unknown-context fallback，并在 `record_assertion_subject_pid` 接线后才宣称进程维度准确 || OQ-032 | daemon 当前兼容循环仍逐断言扫描，蓝图目标是 `(pid, kind)` 倒排索引和单事件 <3µs | 过早引入共享索引可能破坏单写者和回放确定性 | 默认先用纯 `AlignmentEngine` 固化 golden/fuzz，再在 G3 bench PR 中引入事件循环内倒排索引 || OQ-033 | 蓝图要求 ratatui 完整交互 timeline，但当前工作区未冻结 ratatui 依赖且离线缓存没有该 crate | 直接引入会改变锁文件并阻塞离线构建 | 默认先冻结 renderer/JSON 契约与 ANSI 降级；在 G4 发布门接入 ratatui 后再启用交互循环 || OQ-034 | `query.why` 需要从 `.cllens`/SQLite 查询，而当前 daemon 仅有内存 session | 直接宣称离线查询完成会误导发布验收 | 默认 v0.1 先以 daemon RPC 内存索引背书；G4/G7 增加 `.cllens` 打开与索引回归，缺失时报告明确错误 || OQ-035 | 蓝图要求 `CreateFileMappingW(SEC_COMMIT)` + 命名事件 + Release/Acquire 进度，但当前工作区只有纯 Rust ring 协议契约 | 静态协议测试不能证明跨进程可见性或崩溃恢复 | 默认先冻结 header/95% spool/CRC 契约；G1 Windows runner 接入真实 mapping、通知和恢复测试后才算通过 || OQ-036 | 控制管道要求当前用户 SID ACL；owner-only SDDL 可表达创建者 owner，但尚未在 clean VM 验证 UAC/跨用户拒绝 | ACL 误配会扩大控制面或阻断 GUI/CLI | 默认使用 `D:P(A;;GA;;;OW)`，发布前用真实 SID/跨用户测试确认；失败则改为显式 token SID ACL || OQ-037 | `CL-2001` 三次心跳失联后的自动落盘/停会话尚未由 capture↔daemon runtime 接线 | 仅有计时契约无法证明故障收尾 | 默认先输出健康度和 failure code；G5 接线后要求 spool flush、会话关闭和 CLI/GUI 可见错误的 E2E 证据 || OQ-038 | 本机没有 Azure Trusted Signing 凭据，无法生成合法正式签名 | 不能用自签名证书冒充高等级发布签名 | 默认只生成 `-dev` 包并让正式 gate 阻断；CI 注入 Trusted Signing DLib/metadata 后再发版 |
| OQ-039 | 安装器默认目录与 `%LOCALAPPDATA%\causativelens` 会话根可能重合 | 卸载若递归删除根目录会误删会话 | 默认卸载只删已知安装文件；会话扩展名/目录仅在勾选或 `/CLEANUSERDATA` 时清理，锁定文件移入 `.trash/pending` |
| OQ-040 | NSIS 安装器无法把含反斜杠的自定义路径直接写成 JSON 字符串 | `install-manifest.json` 会产生非法 JSON | 默认 manifest 只写产品/版本/架构/模式和 payload 文件清单，不写未转义绝对路径 |

| OQ-039 | 新增三语要求会扩大 UI/安装器/文档维护面 | 文案漂移或 JSON 契约被误翻译 | 默认已决策：`en-US`、`zh-CN`、`zh-TW`；CLI/GUI/NSIS 各自本地化，JSON/RPC 保持英文稳定，并用架构测试锁住三语入口 |
| OQ-040 | 旧 plan/spec/task-card 文件以中文编写，完全人工翻译会增加契约漂移风险 | 开源读者可能找不到对应语言的规范 | 默认已决策：保留原契约路径与编号，增加三语入口/索引和核心指南镜像；逐文件翻译按文档变更触发，英文镜像不得改写契约语义 |

