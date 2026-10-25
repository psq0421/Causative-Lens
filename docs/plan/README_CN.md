# CausativeLens v0.1.0 方案与任务卡

语言切换：[English](README.en-US.md) · [简体中文](README_CN.md) · [繁體中文](README_TW.md)。

本目录把 `CL-BLUEPRINT-001` 的 M0–M7 转换为同一发布列车的 G0–G7 闸门。v0.1.0 一次性交付 ETW 采集、推理侧接入、对齐、CLI、daemon、18 个目标运行时适配、Tauri 2 GUI 和 x64/ARM64 安装器；原 M8（干预与 Jev 决策）顺延 v0.2.0。v0.1.0 仅 `observe`，不注册干预族/决策族命令。

## 闸门与任务卡总览

| 闸门 | 周期 | 出口判据 | 任务卡 |
|---|---:|---|---|
| G0 基线与安装器骨架 | 1周 | workspace/CI 全绿；空安装包可装卸；版本与命名契约冻结 | T0001–T0005 |
| G1 ETW 采集 | 4周 | 四 provider、手工解码、rundown、spool、健康度；x64/ARM64 构建 | T0010–T0015 |
| G2 推理接入 | 3周 | 18 个目标运行时适配、Harness tail、规则断言池、daemon RPC | T0020–T0023 |
| G3 对齐引擎 | 4周 | 三元对齐、水位线、履约图 SQLite；12 剧本一致 | T0030–T0032 |
| G4 查询与 CLI | 3周 | observe CLI 全量命令、回放、export、doctor；错误码稳定 | T0040–T0041 |
| G5 daemon 与运行时接线 | 3周 | daemon 常驻接线、Named Pipe、19 项运行时 tail/doctor/哨兵 | T0020–T0023、G5-report |
| G6 Tauri GUI | 2周 | 四工作区、一 daemon 两前端、性能红线 | T0050–T0052 |
| G7 安装器与发布门 | 2周 | 双架构安装器、签名、E2E、卸载、20 会话、Release 两 exe | T0060–T0072；当前 x64 本机版已完成，正式签名/ARM64待 CI |

每个闸门均交付当前进度可安装的 x64/ARM64 包；闸门出口引用蓝图表18的原判据并按新名称重述。任务卡位于 `task-cards/`。

## 产出物

- [implementation-plan.md](implementation-plan.md)
- [gui-ux-spec.md](gui-ux-spec.md)
- [installer-spec.md](installer-spec.md)
- [gate-reports/G7-report.md](gate-reports/G7-report.md)：x64 安装器、签名阻断与 E2E 闸门报告
- `installer/README.md`：NSIS 构建、Trusted Signing、SBOM 与 E2E 操作手册
- [test-plan.md](test-plan.md)
- [open-questions.md](open-questions.md)
- [architecture-audit-3.md](architecture-audit-3.md)：蓝图 3.1–3.5 总体架构逐项审计
- [architecture-stack-audit.md](architecture-stack-audit.md)：Rust、并发、CLI、SQLite/spool 与依赖策略审计
- [data-model-audit-6.md](data-model-audit-6.md)：蓝图第 6.1–6.5 核心数据模型审计
- [../spec/data-model-6.md](../spec/data-model-6.md)：SysEvent、Assertion、Verdict 与 serde/存储契约
- `task-cards/`：按附录 A 模板拆分的 T0001–T0072 任务卡
- `task-cards/T0080-总体架构复检.md`：蓝图第 3.1–3.5 复检与补充记录
- `task-cards/T0081-技术栈与并发复检.md`：Rust、线程、CLI、存储与依赖治理复检
- `task-cards/T0082-核心数据模型复检.md`：蓝图第 6.1–6.5 的 SysEvent、Assertion、Verdict、稳定 ID 与路径驻留复检

- [etw-audit-7.md](etw-audit-7.md)：蓝图第 7.1–7.11 Windows ETW 采集审计
- [../spec/etw-7.md](../spec/etw-7.md)：会话、权限、keyword、解码、rundown 与健康度契约
- `task-cards/T0083-ETW采集第7章复检.md`：蓝图第 7.1–7.11 的 ETW 采集复检与真实 runner 门禁

- [harness-audit-8.md](harness-audit-8.md)：蓝图第 8.1–8.2.3 Harness 适配、quarantine、规则/语义断言池审计
- [../spec/harness-8.md](../spec/harness-8.md)：JSONL tail、目录通知、未知字段、断言提取与语义层契约
- `task-cards/T0084-Harness适配与断言提取复检.md`：第 8 章复检与 G5 真实 fixture 前置门禁
- [alignment-audit-9.md](alignment-audit-9.md)：蓝图第 9.1–9.9 因果对齐引擎复检
- [../spec/alignment-9.md](../spec/alignment-9.md)：评分、时间窗、水位线、路径、网络和进程归属契约
- `task-cards/T0085-因果对齐引擎第9章复检.md`：第 9 章复检与 G3 对齐门禁

- [query-11.md](../spec/query-11.md)：`why` 受限路由、五段式查询、回放叠加与终端渲染契约
- [query-audit-11.md](query-audit-11.md)：第 11 章查询/回放/终端复检记录
- `task-cards/T0086-查询回放与终端渲染复检.md`：G4 查询与回放补齐任务卡
- [ipc-security-12.md](../spec/ipc-security-12.md)：ring、控制协议、ACL、心跳与攻击面契约
- [ipc-security-audit-12.md](ipc-security-audit-12.md)：第 12 章 IPC/安全复检记录
- `task-cards/T0087-IPC与安全模型复检.md`：G5 IPC 与安全接线任务卡

