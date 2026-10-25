# 架构总览

> **daemon 是心跳，两个前端是它向世界打开的窗。**

CausativeLens 采用双进程观测拓扑：`clens-capture`（UAC、最小特权面）采集 ETW，通过 mmap ring、`.clspool` 和控制管道把事件交给标准用户权限的 `clens-daemon`；daemon 负责 Harness 适配、断言提取、对齐、水位线、归因、SQLite 与查询，再通过 `causativelens-rpc` 同时服务 `clens.exe` 与 `clens-gui.exe`。

```text
ETW provider → clens-capture → mmap ring/.clspool → clens-daemon
                                      ├→ SQLite .cllens / query
                                      └→ causativelens-rpc → CLI + GUI
```

核心承诺是“一 daemon，两前端”：GUI 和 CLI 不能直链引擎 crate，必须向同一个 daemon 请求同一份 observe-only 证据。只有 capture 提权；v0.1.0 不装服务、不加载驱动、不注入，也没有干预通道。

`SysEvent`、`Assertion`、`Verdict` 是 `clens-types` 中带 schema 版本的纯数据。ETW QPC、UTC 墙钟与 Harness UTC 通过锚点统一，水位线在乱序和断言滞后预算安全后才关闭窗口。SQLite 保存索引与薄履约图，高频明细留在压缩 spool 块；崩溃后按 `seq` 续读并去重。

低权限进程只能发送白名单控制协议；capture 不接受策略语义、不读用户输入、不联网、不执行命令。解码器遇到坏记录会拒绝而不是 panic，JSONL 与模型输出进入 fuzz/quarantine。

阅读[总体架构审计](../plan/architecture-audit-3.md)、[IPC 契约](../spec/ipc-security-12.md)和[数据模型](../spec/data-model-6.md)。语言：[English](README.md) · [简体中文](README_CN.md) · [繁體中文](README_TW.md)。
