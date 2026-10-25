# 技术栈与运行时架构审计

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

审计依据是本轮提交的 Rust/并发/CLI/存储要求，以及已经批准的 v0.1.0 方案。结论区分“已补齐契约”“已有实现”“仍需实现”。

## 结果

| 主题 | 当前状态 | 处理 |
|---|---|---|
| Rust 核心 | 所有 workspace crate 都是 Rust；GUI 也只有 Tauri 壳 | 保持 Rust 作为核心实现语言。 |
| 静态 CRT | 原来没有 target 配置 | 新增 `.cargo/config.toml`，x64/ARM64 MSVC 请求 `+crt-static`；需在 Windows release 链接阶段实测。 |
| windows-rs/bytemuck | 当前 capture 仍用 `logman` 命令和 JSON fixture 解码，尚未接 StartTrace/TDH typed API | 不能宣称 ETW typed decoder 已完成；列入 G1 阻塞项，后续以 feature-gated `windows`/`bytemuck` 实现。 |
| 固定线程模型 | 原来没有线程计划，也没有真实 worker 启动 | 新增 capture 六线程计划和 daemon 线程预算函数/契约测试；实际 ProcessTrace、spool、alignment、SQLite worker 仍需接线。 |
| crossbeam/ArcSwap | 当前没有引入 | 本轮不伪造依赖已落地；crossbeam 通道、分片结构和配置热替换列为并发实现卡。 |
| Tokio | 当前未进入 workspace | 保持不引入全局 Tokio；适配层后续若需要异步 IO，只能 feature-gate。 |
| CLI | 当前有 observe 命令集合和 daemon RPC fallback，但解析器仍是手写，`shell` 尚未成为 REPL | 已保留 CLI 优先原则；clap v4/REPL 列为 G4 实现项。 |
| GUI 时间 | 新要求描述 v1 GUI，已批准方案要求 v0.1.0 G6 GUI | 按已批准方案交付 v0.1.0 GUI；将“CLI 优先”解释为核心路径和交互顺序优先，不移除 G6。 |
| 包体/RSS | 新要求 `<15MB`/`<120MB`，已批准 R7 是 `<25MB`/`<150MB` | v0.1.0 保留 `<25MB`/`<150MB` 硬门；`<15MB`/`<120MB` 作为优化目标，需人类改红线后才升级。 |
| SQLite | 当前只有 DDL 常量和 spool writer，无 rusqlite writer | 已补 FTS5 schema 契约；rusqlite bundled、单写者线程和 `.cllens` writer 仍未实现。 |
| 列式 spool | 当前为 CRC append frame，不是 zstd 压缩列式块 | 保留 `.clspool` 兼容帧；列式 block、zstd 压缩和 block index 作为存储卡，不把当前帧格式误称为最终列式实现。 |
| 依赖治理 | 原来没有 deny/audit 配置 | 新增 `deny.toml`、`dependency_audit.ps1` 和依赖策略契约，并加入 CI。工具未安装时只输出降级警告，不能作为发布通过证据。 |

## 并发预算契约

capture 计划为四个 provider 消费线程、一个 spool writer 和一个控制线程。daemon 计划为 spool reader、Harness tail、`clamp(physical_cpus / 2, 1, 4)` 个断言 worker、一个对齐线程和一个 SQLite writer。线程计划目前是可测试的预算边界；没有为未接线线程伪造运行态完成。

## 冲突与默认调和

1. “v0.x 纯 CLI、v1 GUI”与批准的 v0.1.0 G6 GUI 冲突。默认采用批准方案，GUI 保持 v0.1.0；CLI 优先表示 Rust 核心和命令语义先稳定。
2. `<15MB`/`<120MB`与 R7 `<25MB`/`<150MB`冲突。默认采用 R7 硬门，新增数字作为 stretch target。
3. “SQLite + zstd 列式 spool”与当前未引入 rusqlite/zstd 冲突。默认先保持 `.clspool` CRC 帧兼容，再单独落地列式 block 和 SQLite writer，避免静默改格式。

## 尚未通过的验收

Windows `windows-rs` ETW/TDH、零拷贝 cursor、静态 CRT release 链接、crossbeam 实际线程、rusqlite FTS5、zstd 压缩吞吐、clap REPL、包体/冷启动/RSS，均需要后续实现和实测证据。

