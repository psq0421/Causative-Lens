| <br>C  A  U  S  A  L  L  E  N  S     D  E  V  E  L  O  P  M  E  N  T     B  L  U  E  P  R  I  N  T<br>CausativeLens<br>技术开发蓝图<br>Agent 推理-执行因果对齐器 · 完整工程开发规格书<br>版本 v0.x（M0–M5 全周期） · Windows 优先 · Rust 实现<br>文档编号 CL-BLUEPRINT-001<br>2026 年 10 月 24 日 · Approved for development<br><br>MIT Open Source                                        CL-BLUEPRINT-001 / v1.0 |

| --- |

目  录

（打开文档后请右键点击目录 → 更新域 → 更新整个目录，以刷新页码）

文档信息

文档性质：工程开发规格书（Development Blueprint），开发与验收的单一权威来源。

适用版本：CausativeLens v0.x（M0–M5 全周期）。第 18 章与 v2 相关。

维护规则：本文档与仓库 docs/spec/ 契约、契约测试三者保持同步，冲突时以测试为准（见附录E 文末说明）。

阅读路径建议：人类开发者按章顺序读；Codex 类代理从附录A开始；评审者第 3、7、9、13 章是重点。

文档编号：CL-BLUEPRINT-001 · 生成日期 2026-10-24 · 状态：Approved-for-development

## 第1章  执行摘要

CausativeLens 是一个面向 Agent 开发者的推理-执行因果对齐器。它不监控 Agent 做了什么，而是把 Agent 推理轨迹中的每一步编译成可验证的因果断言（例如「我将只读方式访问 config.yaml」「失败则回滚」），再在操作系统级的真实行为轨迹（Windows 上由 ETW 采集的进程、文件、网络、注册表事件）中逐条验证这些断言是否被忠实执行，最终输出一张因果履约图：每条断言标记为已履约、偏离或未观测到，每个偏离点被高亮为因果断点。

本文档是 CausativeLens 的完整工程开发蓝图，覆盖系统架构、数据模型、六个子系统的实现规格、性能预算、测试策略、里程碑任务分解与 AI 编码协作（Codex）规范。所有规格细化到模块接口签名、关键 API 调用序列、算法伪代码与验收命令级别，可直接作为开发与验收依据。

全文第一设计约束是运行效率：核心路径全部采用 Rust 实现，采集进程与引擎进程分离，事件解码走零拷贝手工布局路径而非通用动态解析，对齐引擎按增量水位线流式执行，常驻内存有硬上限。全部 GUI 方案（Electron 一类）被明确排除，v0.x 交付 CLI 优先形态，图形界面后置为基于系统 WebView 的轻量方案。

表1  CausativeLens 核心交付物一览

| 交付物 | 形态 | 核心价值 |

| --- | --- | --- |

| clens-capture 采集器 | Windows 特权进程（Rust 原生） | ETW 多 provider 会话管理、事件解码与顺序化 spool |

| clens-harnessd 适配器 | 非特权进程（Rust） | tail Harness append-only 事件流，提取因果断言 |

| clens-core 对齐引擎 | 非特权进程（Rust 库 + 可执行） | 断言-事件三元对齐、履约图构建 |

| clens-query 查询层 | CLI（Rust） | 自然语言/DSL 查询、因果链回溯、回放叠加 |

| .cllens 会话文件 | SQLite + 列式 spool 容器 | 自包含、可归档、可离线复盘的观测记录 |

| 履约图（CausativeFulfilmentGraph） | 结构化数据 + 文本渲染 | 断言级、事件级的因果判定与断点定位 |

一句话总结本蓝图的立场：CausativeLens 补的是「推理层承诺」与「系统层事实」之间的验证层，它不替代任何 Agent 运行时，也不替代任何 EDR/行为审计工具；它的唯一核心对象是跨层的履约关系本身。

## 第2章  问题定义与产品边界

### 2.1  要解决的工程问题

当前调试一个 Agent 时，开发者面对两条互不相干的轨迹。第一条在推理侧：Harness 的 append-only 事件流记录模型每一步的推理文本与工具调用意图，例如「第 7 步推理说：需要读取 config.yaml 以获取端口」。第二条在系统侧：ETW 事件流记录进程 A 在 T+3.2s 对 config.yaml 发起了 ReadFile，随后对 8080 端口发起了 bind。现有工具链没有任何一个组件把这两条轨迹焊接在一起并回答：第 7 步的推理意图，是否被后续的系统行为忠实执行了。

这个盲区造成三类典型且致命的调试失败。第一类是访问模式违约：推理声称只读，系统轨迹显示发生了写。第二类是时序倒置：推理声称检查完成后再删除，但删除的时间戳早于检查。第三类是参数溯源断裂：推理声称某个参数来自用户输入或某个文件，但系统调用中的实际参数值对不上。行为审计工具能看到「写了文件」，但看不到「这次写是否违背了同一会话中推理层做出的承诺」。CausativeLens 的全部设计都围绕把这一层验证自动化、可查询、可回放展开。

### 2.2  与现有工具的本质差异

以下对比基于对各项目公开定位的理解；CausativeLens 不贬低任何现有项目，只划清抽象层边界。

表2  同类工具抽象层对比

| 工具/项目 | 核心抽象 | 是否验证推理层声明 |

| --- | --- | --- |

| numbat / Aegis 类行为审计 | 行为模式匹配（做了什么） | 否：无推理侧输入 |

| agent-edr 类归因工具 | 工具调用到进程的映射（谁做的） | 否：不做声明级验证 |

| ExecWeave 类执行溯源 | 类型化执行图、跨层 provider 语义 | 部分：偏工具调用语义，推理承诺非一等公民 |

| 传统 EDR / Sysmon | 系统行为审计与告警 | 否：不接入推理轨迹 |

| CausativeLens | 推理声明到系统事实的履约关系 | 是：履约关系是唯一核心对象 |

差异的本质在于：行为审计问「系统发生了什么」，归因工具问「是哪个工具调用触发的」，CausativeLens 问「推理层做出的每一条承诺，系统层是否兑现了」。第三个问题的答案天然是一张图而不是一张表，因此 CausativeLens 的输出物从第一天起就按图结构设计。

### 2.3  目标用户与运行环境

首要平台：Windows 10 1809+ / Windows 11，x64 与 ARM64。采集子系统深度绑定 ETW，Windows 是 v0.x 唯一完整实现平台。

跨平台承诺方式：系统侧接口抽象为 SysEventSource trait。Linux 计划走 eBPF（bpftrace/libbpf 路线），macOS 计划走 Endpoint Security 框架，均为 v1 之后路线；对齐引擎、存储、查询层与平台无关，先行落地。

Android 位置：仅规划为只读伴侣端（查看与查询），不承担采集职责；分发预留 F-Droid 通道，不进 Google Play。

目标用户：在 Windows 上运行 Harness 类 Agent（如 DeepSeek Harness 及任何暴露结构化推理轨迹的运行时）的开发者与高级玩家。用户默认具备命令行能力。

开源协议：MIT，仓库与发布物保持协议层干净，无 copyleft 依赖（详见 4.5 依赖准入）。

### 2.4  非目标（明确不做的事）

不做 Agent 运行时：不执行模型推理、不管理工具注册表、不替代 Harness。

不做通用 EDR：不做恶意行为检测规则引擎、不做威胁情报、不做阻断。CausativeLens 全程只读观测，永远不对系统执行干预。

不做 NIDS/网络内容分析：网络侧只关心 connect/bind/listen/send/recv 的元数据与目标地址，不解载荷。

v0.x 不做语义级源码理解：不解析被观测进程的代码语义，归因完全基于事件流、进程树与断言匹配。

不承诺零漏报：ETW 有固有的缓冲丢失与可见性边界（详见 7.9），履约状态中的「未观测到」就是对这些边界的诚实表达，而不是缺陷。

边界声明  所有对外文档与 README 必须包含上述非目标清单。这是产品边界管理的一部分：CausativeLens 的价值来自聚焦，模糊边界会把它拖进 EDR 红海。

## 第3章  系统总体架构

### 3.1  顶层架构决策

CausativeLens 采用双进程架构：一个最小特权面的采集进程（clens-capture，需要管理员令牌）和一个非特权的主进程（clens，包含适配器、对齐引擎、存储与查询）。两者通过一个内存映射环形日志加磁盘 spool 的混合通道通信。这个拆分是全文档最重要的单项架构决策，理由如下。

特权面最小化：ETW 内核 provider 会话需要管理员权限。把需要提权的代码压缩到一个职责单一、代码量小（目标 < 4000 行）、可被完整审计的进程里，攻击面与事故面都可控。

稳定性隔离：对齐引擎、LLM 断言提取、SQLite 写入都不允许拖垮采集。采集进程只做一件事：把 ETW 事件解码后顺序化写出。它崩溃或被杀时，spool 文件仍在，主进程重启后从断点续读，数据语义是 at-least-once 加序列号去重。

性能隔离：采集线程的热路径（解码、FileObject 表维护）不受引擎 GC 式停顿影响（Rust 本无 GC，这里隔离的是逻辑抖动：批量 flush、锁竞争、页错误风暴）。

### 3.2  进程拓扑与数据流

表3  CausativeLens v0.x 进程拓扑

| 组件 | 特权级 | 职责 | 上游输入 | 下游输出 |

| --- | --- | --- | --- | --- |

| clens-capture | 管理员 | ETW 会话生命周期、事件解码、FileObject→路径解析、顺序化 | ETW 实时会话（4 个 provider） | mmap ring → spool 文件（.clspool） |

| clens（主进程） | 标准用户 | 调度 capture、tail Harness 日志、断言提取、对齐、存储、查询 | spool 文件、Harness JSONL | SQLite（.cllens）、CLI 输出 |

| 断言提取器 | 主进程内线程池 | 规则层抽取 + 本地小模型裁剪 | Harness JSONL 事件 | 断言流（Assertion IR） |

| 对齐引擎 | 主进程内 | 时间轴合并、三元对齐、履约图 | 系统事件流 + 断言流 | 履约图写入 SQLite |

| 查询层 | 主进程内 | DSL/NL 查询、因果链回溯、回放叠加 | .cllens 文件 | 终端渲染 / JSON 导出 |

端到端数据流共八站：(1) ETW provider 产生内核事件；(2) capture 进程的四个消费线程分别处理四个 provider 会话；(3) 手工解码器把 EVENT_RECORD 转成统一事件 IR，期间维护 FileObject 到路径的映射表；(4) 事件按全局序列号写入 mmap ring，ring 满时溢写磁盘 spool；(5) 主进程读入事件，与 Harness 断言流汇合于统一时间轴；(6) 对齐引擎按水位线增量执行三元对齐；(7) 履约图与原始事件批量写入 SQLite；(8) 查询层在会话文件上执行查询与回放叠加。

### 3.3  时钟域与全局时序

系统里存在三个时钟：ETW 事件时间戳（QPC 域，单调、高分辨率）、Windows 墙钟（UTC）、Harness 事件时间戳（UTC，毫秒级）。对齐的前提是把三者放进同一个坐标。方案如下。

锚定：采集启动瞬间，capture 连续做 16 轮「同拍采样」：每轮先读 QueryPerformanceCounter，再读 GetSystemTimePreciseAsFileTime，再读一次 QPC，取 QPC 中值与 UTC 配对；16 轮取配对间隔最小的一组，得到 (qpc0, utc0) 锚点。理论残差在几十微秒量级。

持续校准：主进程每 60 秒产生一次双向心跳——写一条注入 Harness 可见日志的心跳事件，同时 touch 一个专用哨兵文件（触发一次可识别的 ETW 文件事件）。两个域各自记录的心跳配对后估计漂移，漂移超过 200µs/s 时刷新锚点并把校准事件写入会话文件。

Harness 侧延迟吸收：Harness 事件的时间戳是写入时刻而非发生时刻，天然带毫秒级噪声。对齐层不为 Harness 侧追求高精度，而是把它当作一个带置信区间的观测点处理（见 9.5 评分函数的时间项设计）。

全局顺序：跨 provider 的事件顺序以 ETW QPC 时间戳为主序。多 CPU buffer 引发的乱序由重排序缓冲处理（默认 64ms 窗口，见 7.6），spool 内事件保证 QPC 单调不减。

### 3.4  一次典型会话的时序

以「用户调试一个读改 config.yaml 的 Agent」为例：(1) 用户执行 clens run -- agent.cmd；(2) clens 以 UAC 提权启动 capture，完成 ETW 四会话建立与 rundown；(3) clens 拉起 Agent 进程（作为 clens 的子进程，从而进入进程树观测范围）；(4) capture 流式吐出系统事件；(5) Harness 每完成一步推理就 append 一行 JSONL；(6) 断言提取器在 1 秒批延迟内产出断言；(7) 对齐引擎增量匹配；(8) Agent 结束后 clens 冲刷全部流水线，输出终版履约图摘要并落盘会话文件。全程用户可随时用 clens why 断言查询当前已判定的因果链。

### 3.5  部署形态

嵌入式运行（clens run）：CausativeLens 主动拉起被测 Agent，观测窗口天然覆盖进程全生命周期，是默认形态。

挂载式运行（clens attach --pid）：附加到已运行进程树，依赖 rundown 补齐已存在的进程与句柄；语义上观测从 attach 时刻开始，此前的行为不在因果判定范围内（在输出中显式标注观测起点）。

离线复盘（clens open session.cllens）：完全脱离采集，只读会话文件做查询与回放，可在无管理员权限的机器上进行。

## 第4章  技术选型与论证

### 4.1  语言：Rust（核心）

全核心路径使用 Rust，理由按权重排序：

性能确定性：无 GC 停顿、无运行时抖动，p99 延迟可承诺。事件解码热路径上每一次堆分配都要过代码评审，这在 GC 语言里是做不到的纪律。

零拷贝与显式内存布局：ETW 事件是 packed C 结构，Rust 的 &[u8] cursor 解析 + bytemuck 零拷贝转换是天然匹配；对 mmap ring 的 volatile 读写与内存序控制表达力完整。

Windows API 一等公民：windows-rs（微软官方维护的 Rust 绑定）覆盖 ADVAPI32 的 StartTrace/EnableTraceEx2 与 TDH 全套 API，类型安全且持续跟进。

交叉能力：同一 codebase 以 cfg(target_os) 分支覆盖未来 Linux(eBPF)/macOS(ES) 采集端，业务层零改动。

二进制形态：静态链接 CRT，单 exe 冷启动 < 50ms，安装包 < 15MB。这是对 Electron 类方案的根本性反制。

被否决的备选：C++（同等性能但内存安全成本在长周期项目中不可接受，且 windows-rs 的类型安全收益巨大）；Go（GC 停顿 + 调 Win32 需绕行系统调用封装，二进制 10MB 起）；Zig（生态未成熟，windows 绑定不全）；C#/.NET（AOT 可行但 ETW 热路径的分配纪律难维持，且自宿主更新链路重）。

### 4.2  运行时与并发模型

关键决策：不引入 Tokio 全家桶作为全局运行时。核心引擎（解码、对齐、存储写入）使用传统 std::thread + crossbeam 通道 + 无锁/分片数据结构；只有适配层的文件 tail、子进程管道 IO、CLI 交互这类真正的异步 IO 少量使用 tokio（或直接用 std 线程阻塞读，量级小到不值得引入依赖）。理由：Tokio 的调度器、任务装箱与 wakeup 机制在微秒级热路径上是纯开销；本项目并发模型是「固定少量线程 + 显式通道」，不是海量并发任务，异步运行时解决的不是我们的问题。

capture 进程：4 个 ETW 消费线程（每 provider 一个 ProcessTrace 调用各占一个线程）+ 1 个 spool 写线程 + 1 个控制线程。

主进程：1 个 spool 读线程、1 个 Harness tail 线程、N 个断言提取 worker（N = 物理核数的一半，上限 4）、1 个对齐线程（单线程事件循环，见 9.7）、1 个 SQLite 批量写线程。

线程间数据交换全部显式：crossbeam 的 SegQueue / channel + ArcSwap 配置热替换。禁止任何组件在自己线程里直接碰 SQLite 连接（单一写者原则）。

### 4.3  UI：CLI 优先，系统 WebView 后置

v0.x 是纯 CLI（clap v4 构建，含交互式 REPL 子命令 clens shell），性能敏感路径零 UI 开销。v1 的图形界面规划使用 Tauri 2（Rust 核心 + 系统 WebView2 渲染层，Windows 上复用系统自带 WebView2 Runtime，不打包浏览器内核），安装体积目标 < 25MB、常驻内存目标 < 120MB。Electron 被硬性排除，量级参考：Electron 空壳应用安装包 > 85MB、常驻 RSS > 250MB、冷启动 > 800ms——与本项目「观测工具本身必须比被观测对象更轻」的原则直接冲突。

原则  观测工具的资源占用会污染被观测轨迹（CPU 竞争影响调度时序、内存压力影响系统行为）。工具自身的 CPU/RSS/IO 占用本身就是设计指标，见 13.1 性能预算表。

### 4.4  存储：SQLite + 自定义列式 spool

会话级持久层：SQLite（WAL 模式，rusqlite 绑定），承载断言、履约图、事件索引与全文检索（FTS5）。选择理由：单文件自包含、跨平台可携、工具链成熟（cli 可用 sqlite3 直接开箱）、写吞吐经调优后足以覆盖本项目事件率（实测目标见 13.1）。

高频事件原始层：磁盘 spool 采用自定义紧凑二进制列式分块格式（.clspool，帧格式见 10.3），按块 zstd 压缩。原始事件不逐条进 SQLite 行——这是把 SQLite 用对的关键：SQLite 存索引与判定，列式块存明细，查询时按需解码明细块。

被否决：DuckDB（分析强但单写并发与嵌入形态的稳定性在 Windows ARM64 上未验证）；纯自定义格式（放弃 SQLite 的查询生态与 FTS5 不划算）；Postgres（部署形态完全不符）。

### 4.5  依赖准入红线

所有第三方 crate 必须满足：MIT/Apache-2.0/BSD/ISC/MPL-2.0 协议（排除 AGPL/SSPL/未标注）；有持续维护（6 个月内 commit）；无 unsafe 泛滥（unsafe 行数占比过高或审计不过直接弃用）；Windows CI 绿。核心链路依赖白名单（初始集）：windows、rusqlite（bundled SQLite）、zstd、crossbeam、clap、serde/serde_json、tracing、thiserror、bytemuck、dashmap、globset、ariadne（错误渲染）。新增依赖需在 PR 中声明理由并过 CI 依赖审计（cargo audit + cargo deny）。

## 第5章  代码仓库结构与工程规范

### 5.1  Cargo Workspace 布局

CausativeLens/

├── Cargo.toml               # workspace: 只列成员与共享 profile

├── crates/

│   ├── clens-types/         # 全部公共 IR: SysEvent / Assertion / Verdict ...

│   ├── clens-capture/       # 特权采集进程 (Windows ETW, bin)

│   │   └── src/etw/         # session.rs / decode/ / fileobject.rs / rundown.rs

│   ├── clens-harness/       # Harness 适配器: tail 协议 + 事件 schema

│   ├── clens-extract/       # 断言提取: 规则层 + LLM 兜底层

│   ├── clens-align/         # 对齐引擎 (纯计算, 无 IO)

│   ├── clens-store/         # spool 读写 + SQLite schema + FTS5

│   ├── clens-query/         # DSL 解析 / NL 路由 / 渲染 / 回放叠加

│   ├── clens-ipc/           # mmap ring 协议 + spool 帧格式 + 心跳

│   └── clens-cli/           # bin: clens (run/attach/open/why/...)

├── xtask/                   # cargo xtask: new-task / fixture / bench 母命令

├── fixtures/                # 合成 ETW 事件流 / Harness JSONL 回放数据

├── tasks/                   # T00xx-*.md 任务卡 (Codex 的作业面, 见附录A)

├── docs/spec/               # 每个 crate 的接口契约 spec.md

└── .github/workflows/ci.yml # fmt + clippy + test + audit + bench-smoke

### 5.2  Crate 职责与依赖方向

表4  crate 依赖图（箭头表示 depends on）

| crate | 依赖 | 禁止依赖 | 发布形态 |

| --- | --- | --- | --- |

| clens-types | serde, bytemuck | 任何平台/IO crate | lib |

| clens-ipc | clens-types, zstd, windows(共享内存API) | clens-capture | lib |

| clens-capture | clens-types, clens-ipc, windows | clens-store, 对齐相关 | bin (提权) |

| clens-harness | clens-types, serde_json, notify | tokio 全家桶之外的异步 | lib |

| clens-extract | clens-types, globset, ort(可选) | clens-align | lib |

| clens-align | clens-types, dashmap, crossbeam | 任何 IO、任何数据库 | lib |

| clens-store | clens-types, clens-ipc, rusqlite | clens-align | lib |

| clens-query | clens-store, clens-types | clens-capture | lib |

| clens-cli | 以上全部 | — | bin |

架构测试  ci 中运行 arch-test：用 cargo metadata 解析依赖图并断言上述禁止边不存在。破坏分层的 PR 直接红灯。clens-align 保持纯计算（无 IO）是 fuzz 与基准可复现的前提。

### 5.3  错误处理规范

统一 thiserror，错误类型按 crate 划分，每类错误带稳定错误码（CL-XXYY，见附录B）。禁止裸 unwrap/expect 于任何非测试路径（CI 用 clippy lint restriction 强制：unwrap_used / expect_used / panic）。跨进程边界的错误必须可机读：capture 与主进程的控制通道消息里错误一律携带错误码 + 上下文键值对，CLI 渲染时翻译为人话并给出用户可执行的动作建议（例如权限不足时给出提权命令原句）。

// clens-types/src/error.rs (节选)

#[derive(Debug, thiserror::Error)]

pub enum CaptureError {

#[error("etw session start failed: provider={provider} hr=0x{hr:08X}")]

EtwSessionStart { provider: ProviderKind, hr: u32 },

#[error("buffer loss detected: lost={lost} seq_hole=[{first}..{last}]")]

BufferLoss { lost: u64, first: u64, last: u64 },

// 错误码由 thiserror 枚举序号稳定映射, 见附录B

}

### 5.4  日志与可观测性

tracing + 非阻塞 writer（tracing-appender 的 rolling 配置，后台线程 flush，级别过滤默认 INFO）。工具自身的日志绝不进入被观测数据通道（不得写被观测目录、不得产生会被采集器捕获的高频 IO——自身日志固定写入 %LOCALAPPDATA%\CausativeLens\logs，并在 ETW 侧把该目录加入自身噪声过滤器，见 7.8）。运行期指标（事件吞吐、ring 水位、队列深度、对齐延迟 P50/P99、SQLite 写入延迟）每 5 秒快照进内部 metrics 线程，clens stats 子命令实时查看，会话结束时存入 SQLite 的 metrics 表。

### 5.5  配置体系

TOML 配置三级覆盖：内置默认 → %LOCALAPPDATA%\CausativeLens\config.toml → 命令行 flag（clap 与 figment 集成）。全部配置项见附录C。配置变更需带 schema 版本号，升级时迁移。所有时间窗参数（乱序容忍、对齐窗口、心跳周期）集中在一个 TimingProfile 段，禁止散落硬编码。

### 5.6  CI 门禁（全部任务必须绿灯才能合并）

表5  CI 流水线（windows-latest + windows-11-arm 双矩阵）

| 门 | 命令 | 阈值/规则 |

| --- | --- | --- |

| 格式 | cargo fmt --all -- --check | 零 diff |

| 静态 | cargo clippy --workspace --all-targets -- -D warnings | 零 warning（含 unwrap_used 等 restriction 集合） |

| 单元 | cargo test --workspace | 全绿；覆盖率以 cargo-llvm-covers 计，新增代码行覆盖 >= 80% |

| 架构 | cargo xtask arch-test | 依赖图断言通过 |

| 契约 | cargo test -p <crate> --test contract | 每个 crate 的 spec.md 契约测试通过 |

| 模糊 | cargo fuzz 15min 定时任务（对 decode/align） | 无 panic/泄漏（LibFuzzer + ASan） |

| 基准 | cargo xtask bench-smoke | 吞吐/延迟不低于主分支 95%（回退即红灯） |

| 审计 | cargo audit + cargo deny | 无高危漏洞、无协议违规 |

## 第6章  核心数据模型

### 6.1  设计原则

所有跨模块数据都收敛为三个一等公民：系统事件（SysEvent）、因果断言（Assertion）、履约判定（Verdict）。三者由 clens-types 统一定义，serde 序列化版本化（每个结构带 schema_v 字段，破坏性变更升 major）。全部结构为 plain data：不含行为方法、不含平台类型，保证对齐引擎可离线重放。

### 6.2  系统事件 IR（SysEvent）

// clens-types/src/event.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]

pub enum SysKind {

ProcStart, ProcStop, ThreadStart, ThreadStop, ImageLoad,

FileCreate, FileRead, FileWrite, FileClose, FileRename, FileSetInfo, FileDelete,

RegOpen, RegQuery, RegSet, RegDelete,

NetConnect, NetAccept, NetDisconnect, NetSend, NetRecv, NetListen, NetBind,

}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct SysEvent {

pub schema_v: u16,

pub seq: u64,          // capture 全局序列号 (spool 内单调)

pub ts_qpc: u64,       // QPC 时间戳 (100ns 单位)

pub pid: u32,

pub tid: u32,

pub kind: SysKind,

pub payload: EventPayload,

}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum EventPayload {

Proc { ppid: u32, image: Box<str>, cmd: Box<str>, exit_code: Option<u32> },

File { path: PathIdx, desired_access: u32, io_len: Option<u32>,

write_attrs: bool },

Net { proto: NetProto, local: Endpoint, remote: Endpoint },

Reg { key: Box<str>, value: Option<Box<str>> },

None,

}

// PathIdx: 字符串驻留表句柄 — 热路径上路径只存 u32 下标,

// 驻留表在 capture 侧维护, spool 落盘时按块重写为偏移

pub type PathIdx = u32;

两个关键决策：其一，路径字符串驻留（string interning）。文件事件中同一文件的路径会重复出现成千上万次，热路径上携带 PathIdx（u32）而非字符串，驻留表按块持久化，解码侧一次性物化。基准目标：路径重复率 99% 时解码吞吐提升 3 倍以上。其二，FileRead/FileWrite 的 payload 只携带长度不带缓冲内容——CausativeLens 观测元数据，不碰数据内容（法律与性能双重边界）。

### 6.3  因果断言 IR（Assertion）

断言是推理轨迹中被裁剪出的可验证因果声明。设计上断言不等于整段思维链——它是从推理文本与工具调用中抽出的、形式受限的谓词。形式受限是整个系统能落地的原因：只有把断言约束到有限谓词空间，系统事件的匹配才可能是机械的、可审计的。

// clens-types/src/assertion.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum AssertionOp {

Read, Write, Create, Delete, Rename, Exec,

Connect, Bind, Listen,

RegRead, RegWrite,

AssertPrecondition,   // 我假设 X 为真

AssertPostcondition,  // 完成后 Y 应成立

RollbackPlan,         // 失败则回滚到 W

Constraint,           // 本次访问将保持只读等约束

}

#[derive(Debug, Clone)]

pub struct Assertion {

pub schema_v: u16,

pub id: u64,                  // 断言全局 ID

pub span: HarnessSpan,        // 来源: step 序号 + JSONL 字节区间

pub op: AssertionOp,

pub target: TargetPattern,

pub io_mode: IoMode,          // ReadOnly / ReadWrite / CreateNew

pub expect: Expectation,      // 期望出现 / 期望不出现 / 期望顺序

pub order_key: Option<(u64, u64)>, // 前置/后置断言 ID 对

pub confidence: f32,          // 提取置信度 0..1

pub raw_quote: Box<str>,      // 推理原文引用 (审计用)

}

#[derive(Debug, Clone)]

pub enum TargetPattern {

Path(globset::GlobMatcher),  // 绝对路径 glob, 已规范化

RegistryKey(Box<str>),

Endpoint { host: HostPat, port: Option<u16>, proto: NetProto },

FreeText(Box<str>),           // 兜底: 仅参与弱匹配并降分

}

pub enum Expectation {

MustHappen,          // 已履约 = 观测到匹配事件

MustNotHappen,       // 已履约 = 未观测到 (配合窗口)

HappenAfter(u64),    // 必须晚于断言 id 发生 (时序断言)

}

### 6.4  履约判定（Verdict）与履约图

// clens-types/src/verdict.rs

pub enum Fulfilment {

Fulfilled,        // 观测到与断言一致的系统事件

Deviated,         // 观测到与断言冲突的事件 (因果断点)

Unobserved,       // 窗口关闭仍未观测到 (含可见性边界)

Partial,          // 复合断言部分成立 (如: 读成立, 回滚缺失)

}

pub struct Verdict {

pub assertion_id: u64,

pub state: Fulfilment,

pub evidence: Vec<EvidenceEdge>,   // 指向 SysEvent.seq 的边

pub deviation: Option<Deviation>,  // 断点细节

pub decided_at: Watermark,

}

pub struct EvidenceEdge {

pub event_seq: u64,

pub score: f32,          // 0..1, 匹配强度

pub components: ScoreComponents,

}

履约图的图结构很薄：节点是断言与事件，边只有两类（Evidence 断言→事件，OrderKey 断言→断言）。刻意不引入通用图数据库概念——图以 SQLite 两张表表达（见 10.2），查询层做两跳内的因果链遍历即可满足产品需求，遍历深度作为 v2 可调参数。

### 6.5  ID 与标识规范

seq（系统事件）：capture 进程内原子递增，spool 重排后仍保持单调；主进程重放时以其为幂等键。

assertion_id：主进程内 (harness_log_id, line_no, slot_no) 三元组哈希出的 u64，保证同一日志重放时断言 ID 稳定（回放叠加的前提）。

会话 ID：启动时刻 UTC + 随机后缀，即 session.cllens 文件主键。

## 第7章  采集子系统：Windows ETW 工程规格

### 7.1  会话拓扑

采用每 provider 一条独立实时会话的现代拓扑（弃用 legacy NT Kernel Logger 单会话：它全局唯一、易与系统其他工具冲突、keyword 控制粒度差）。四个内核 provider 各一条会话，会话名前缀 CausativeLens_，启动时若检测到同名残留会话先停后启（崩溃自愈）。

表6  ETW 会话矩阵（v0.x）

| provider | 覆盖事件 | 匹配断言 op |

| --- | --- | --- |

| Microsoft-Windows-Kernel-Process | 进程/线程创建退出、映像加载、含 cmdline 的进程元数据 | Exec、进程树归属 |

| Microsoft-Windows-Kernel-File | 文件 create/read/write/close/rename/setinfo/delete | Read/Write/Create/Delete/Rename、Constraint |

| Microsoft-Windows-Kernel-Registry | 注册表 open/query/set/delete | RegRead/RegWrite |

| Microsoft-Windows-Kernel-Network | TCP/UDP connect/accept/disconnect/send/recv/listen | Connect/Bind/Listen |

GUID 纪律  provider 的 GUID 值在代码中以常量表维护，但这些值必须由测试用 wevtutil ep / logman query providers 的机器输出校验后写入（见 7.10 GUID 自检测试），不得凭记忆或博客摘抄。文档不抄录具体 GUID 值，唯一事实来源是系统 manifest。这是防御性工程：GUID 抄错一位，事件一个都收不到，且没有任何编译期错误提示。

### 7.2  启动与权限

权限要求：管理员令牌。工程路径：clens 主进程以 ShellExecute runas 提权拉起 capture（capture 的 manifest 要求 requireAdministrator，UIAccess=false）；capture 启动后立即降权数据面（写 spool 的目录 ACL 授予 Users 组写权限，控制通道为只读），控制面保留管理员。若用户拒绝 UAC，给出精确错误码 CL-1001 与手动提权命令。

启动序列（伪代码级）：

fn boot() -> Result<(), CaptureError> {

acquire_shutdown_guard();              // 崩溃时下次自愈的墓碑

for p in PROVIDERS {                   // 顺序: Process 先行 (进程树根)

let h = StartTraceW(session_name(p))?,

EnableTraceEx2(h, guid(p),

CONTROL_CODE_ENABLE_PROVIDER,

TRACE_LEVEL_INFORMATION,

keyword_mask(p),               // 见 7.3 keyword 矩阵

0 /*match_any*/, 0 /*match_all*/,

0 /*timeout*/)?;

}

request_rundown(PROCESS);              // 现存进程回枚举, 见 7.7

for s in sessions { spawn_consumer(s); }  // 每会话一个消费线程

write_clock_anchor();                  // (qpc0, utc0) 锚点, 见 3.3

signal_ready_to_parent();

run_control_loop();                    // 停止/心跳/keyword 热调

}

fn shutdown() {

stop_consumers();       // 先停消费再停会话, 保证 flush 完整

for p in PROVIDERS { ControlTraceW(EVENT_CONTROL_CODE_STOP_TRACE, ...) }

flush_spool(); release_guard();

}

### 7.3  Keyword 矩阵与开销控制

每个 provider 只开 Informational 级别 + 白名单 keyword，逐项给出理由（keyword 常量名以 provider manifest 为准，编码进 keyword_mask 函数）：

表7  keyword 白名单（v0.x 默认档）

| provider | 启用 keyword（manifest 常量名） | 为什么不开更多 |

| --- | --- | --- |

| Kernel-Process | WINEVENT_KEYWORD_PROCESS、THREAD、IMAGE | 关闭 CPU 采样类 keyword——与本案无关且高频 |

| Kernel-File | FILE_KEYWORD_CREATE/READ/WRITE/CLOSE/…/RUNDOWN | 不开 DIR、OP_END 类高频低值事件；Default 档可关闭 READ 事件仅留写类（文件读事件量最大） |

| Kernel-Registry | 注册表读写类 keyword | 安全日志类 keyword 不开 |

| Kernel-Network | 连接/断开/收发 keyword | 不开包内容诊断 keyword（hexdump 类） |

文件读事件是全系统吞吐最高的源（一次脚本编译即可产生百万级 ReadFile），因此提供三档采集强度：full（全事件，默认）/ lean（去 FileRead/FileSendRecv 明细，保留 create/write/close 与网络五元组）/ minimal（只有写类与连接类）。档位写入会话 manifest，对齐层读取档位后自动调整未观测到的解释语义（lean 档下 Read 断言的 Unobserved 不计为异常，必须如此，否则会产生系统性误报）。

### 7.4  事件解码：手工布局优先，TDH 兜底

核心性能决策：我们关心的每个事件类型都是 manifest 稳定的 packed 结构。为每个关注的 (provider, opcode/task) 手写零拷贝解码器（&[u8] cursor + bytemuck POD 断言），冷门事件才回落到 TdhGetEventInformation 动态解析。工程依据：TDH 每次调用都涉及 UTF-16 转换与堆分配，热路径吞吐差一个数量级；手工解码器对每个事件类型有单测覆盖的字节布局断言，布局错误会在 fixtures 回放中立即暴露。

// clens-capture/src/etw/decode/file_create.rs (示意)

pub fn decode_file_create(rec: &EVENT_RECORD) -> Result<SysEvent, DecodeError> {

let blob = unsafe { slice::from_raw_parts(

rec.EventHeader().ExtendedDataBufferIfAnyUnsafe() /*占位*/, 0) };

// 真实现: 用户数据区 cursor, 字段序 = manifest 声明序

let mut cur = Cursor::new(user_data(rec));

let file_object = cur.read_u64_le()?;     // FileObject 指针

let _info_class = cur.read_u32_le()?;

let open_trace  = cur.read_u32_le()?;     // 打开意图掩码

let path = intern(cur.read_utf16_z()?)?;  // 驻留表 intern

...

}

mod tests {

// 每个解码器: golden bytes -> golden SysEvent, fixtures 比对

}

### 7.5  FileObject 到路径的解析

关键技术事实：FileRead/FileWrite 事件只携带 FileObject 内核指针与偏移，不含路径。路径必须由 FileCreate 事件建立的 FileObject→路径映射提供，FileClose 时移除。工程实现：

数据结构：按 FileObject 指针值哈希分片的并发表（dashmap，128 分片），读写锁竞争域极小。

生命周期：Create 插入、Close 删除；周期清扫（默认 10s）：未被 Close 覆盖且超过 TTL 的表项向 rundown 机制请求重同步（见 7.7），防止句柄在会话建立前已打开导致的永久幽灵路径。

容量上限：表项上限默认 262144（约 100MB 内存的保守上界），超限走 LRU 驱逐并在 metrics 中计数（驱逐率作为采集健康度指标）。

同一 FileObject 的 Rename 事件会更新路径映射，保证 rename 后的写事件归因到新路径。

### 7.6  乱序与顺序化

ETW 按 per-CPU buffer 写入，消费端时间戳天然乱序。方案：每个消费线程解码后把事件投入小顶重排序缓冲（以 ts_qpc 为键，窗口 64ms），到序或窗口超时即弹出、打全局序列号、写 mmap ring。四个会话各自缓冲，ring 写入端以 crossbeam 的无锁 MPMC 队列汇聚到单写者 spool 线程（单写者保证 spool 内 seq 单调，简化重放）。64ms 窗口的依据：实测 ETW 乱序跨度几乎总在单毫秒级，64ms 是给调度抖动留 60 倍余量的工程值，且可配置（TimingProfile.reorder_window_ms）。

### 7.7  Rundown（存量状态回枚举）

attach 场景必须补齐：已存在进程的进程树（否则进程归属断在半空）、已打开文件句柄（否则 FileRead 没有路径可归因）。工程路径：会话建立后立即以 ControlTrace flush + provider 的 rundown keyword 激活（Kernel-Process 的 rundown 事件会回放全部现存进程的 start 事件，Kernel-File 的 rundown keyword 会回放现存句柄）。rundown 产生的事件带 Rundown 标志位，解码时进 SysEvent 额外字段 is_rundown，对齐层把它视作窗口外的背景事实（不参与时序断言，但参与路径归属）。若 rundown 未开启（run 模式启动太早不存在存量），此步空转即可。

### 7.8  自噪声抑制与过滤器

capture 自身与其子进程的 IO 会被自己采集（自噬）。抑制方式：PID 层过滤器——ETW EnableTraceEx2 的 EnableProperty 支持事件过滤（PASS_FILTER 按进程 ID 过滤的事件头的 Filter 描述符），把 capture 自身 PID、spool/日志目录的 IO 从源头过滤；注意这不是丢失，是显式标注被过滤（会话 manifest 记录 excluded_pids，对齐层在解释 Unobserved 时能区分「被过滤」与「真没发生」）。

主进程（非特权）无法在 ETW 层过滤，其 IO 量小（SQLite 写与终端输出），在引擎层以 PID 路径前缀黑名单软过滤并计数。用户亦可反向操作：--include-only <pid> 只观测目标进程树（默认建议开启，全系统观测是高开销档位）。

### 7.9  丢失、背压与健康度

会话属性中的 EventsLost / RealTimeBuffersLost 每 500ms 轮询，非零即产生 BufferLoss 错误事件进事件流（自身可见），并把健康度降级。

缓冲参数基线：BufferSize 64KB、MinimumBuffers = CPU 核数、MaximumBuffers = 4 倍核数、FlushTimer 1s；事件率超阈值的自动升档逻辑 v1 再做（先把可观测的丢数据做好，再谈自适应）。

ring 水位超过 80% 时 spool 写线程批量加大（合并 4KB→64KB 写粒度）；95% 时丢弃最低价值事件类别（优先弃 FileRead 明细）并记录 drop 计数。丢弃决策表写死在代码中并配套测试，运行期不动态发明策略。

履约语义上的处理：任何观测窗口内发生过丢失的会话，其 Unobserved 判定自动降级为弱判定（输出里标注「采集窗口有丢失，未观测不等于未发生」）。这是产品诚实性的硬规则。

### 7.10  GUID 与解码布局自检

CI 中内置两条自检防线。防线一（GUID 校验）：在一台真 Windows runner 上执行 logman query providers，解析输出断言代码内常量与系统注册的 provider GUID 逐个相等；不相等即红灯，PR 必须修常量。防线二（布局校验）：对每个手工解码器，用 TDH 解析同一 fixture 事件，断言两者产出一致（TDH 作为 golden oracle，手工解码器作为热路径实现）。这保证手工解码的快永远不建立在错上。

### 7.11  版本兼容矩阵

表8  Windows 版本兼容基线

| 能力 | Win10 1809+ | Win11+ | 备注 |

| --- | --- | --- | --- |

| 四 provider 会话 | 支持 | 支持 | 1809 为 ETW 现代 API 基线 |

| Process 事件含 cmdline | 部分 keyword | keyword 完整 | 以 manifest 为准，attach 场景 cmdline 缺失时用 NtQuery 补齐 |

| ARM64 | — | 支持 | CI 双矩阵跑同套测试 |

| 要求提权 | 是 | 是 | 内核 provider 会话硬性要求 |

## 第8章  Harness 适配器与断言提取

### 8.1  适配范围与协议

推理侧输入是任何 append-only 的结构化推理轨迹。v0.x 直接适配 DeepSeek Harness 的事件日志（JSONL，逐行 JSON 对象），同时把适配层抽象成 HarnessAdapter trait，任何能映射到该 trait 的运行时（含自定义 Agent 框架）都能接入。trait 定义：

pub trait HarnessAdapter: Send {

/// 监听起点 (文件/管道/命名通道), 返回 TailHandle

fn attach(&mut self, spec: AttachSpec) -> Result<TailHandle, HarnessError>;

/// 阻塞式拉取下一批原始事件 (内部无异步运行时)

fn next_batch(&mut self, buf: &mut Vec<u8>) -> Result<usize, HarnessError>;

/// 把一行原始 JSON 解析为标准 HarnessEvent (schema_v 化)

fn parse(&self, line: &[u8]) -> Result<HarnessEvent, HarnessError>;

}

pub struct HarnessEvent {

pub schema_v: u16,

pub ts_utc_ms: i64,

pub step: u64,

pub phase: Phase,        // Reasoning / ToolCall / ToolResult / System

pub tool: Option<ToolCall>,

pub text: Box<str>,      // 原文 (推理文本或调用参数)

}

文件 tail 实现：ReadDirectoryChangesW 监听目录变更 + 追加读（记住上次 offset，检测 truncate 即回绕并告警）。JSON 解析用 serde_json，未来热路径可换 simd-json（作为 bench 驱动的优化项，不预先引入）。schema 变化的前向兼容：未知 phase/tool 保留 raw 进库（unknown 类事件不丢），解析失败进 quarantine 队列并在会话健康度里可见。

### 8.2  断言提取流水线（两级）

断言提取是 CausativeLens 与纯日志工具的分界线。流水线两级：规则层（确定性、零成本、覆盖结构化字段）优先，语义层（本地小模型，覆盖自由文本推理）兜底。两级产出都带 confidence 与 raw_quote，判定层用 confidence 调权。

#### 8.2.1  规则层：工具调用的天然结构

Harness 的 tool call 事件本身就是结构化断言源：read(path) 直接编译为 (op=Read, target=Path, io_mode=ReadOnly)；edit(path) 编译为 (op=Write, io_mode=ReadWrite) 加一条隐式前置断言（先读后写顺序）；shell(cmd) 走内置命令模板库（对 rm/mv/cp/curl/ssh 等常见命令的正则模板，模板库以数据文件维护、可用户扩展）。规则层产出 confidence=1.0。模板库覆盖不到的命令进语义层。

#### 8.2.2  语义层：本地小模型裁剪

自由文本推理（「我需要先检查 config.yaml 是否存在端口配置，若不存在则回滚」）必须抽成断言。方案：本地推理，Qwen 系 0.5B–1.5B 级小模型（ONNX Runtime 的 Rust 绑定，int4 量化），输入为裁剪提示词 + 单步推理文本，输出为受限 JSON 断言（语法上用 grammar-constrained decoding 或后校验 + 重试一次）。硬约束：模型只在用户显式启用且本机推理可用时启用（--extractor=llm），离线、无遥测；产出的每条断言 confidence 封顶 0.7，且 raw_quote 必须逐字来自输入（字符串包含校验，杜绝模型幻觉引用）。

批处理：文本按 1s 窗口聚合，单批推理，断言产出的允许滞后（对齐层水位线设计天然容忍，见 9.7）。CPU 推理预算：单步 < 300ms（1.5B int4 在 8 核 CPU 的经验量级），不阻塞任何主路径。

#### 8.2.3  汇聚与去重

两级产出汇入断言池：同 (span, op, target, io_mode) 的完全重复断言合并计数；互斥矛盾（同 span 同 target 先 ReadOnly 后 ReadWrite）不合并而是保留两条并打 contradiction 标记——矛盾本身是重要信号。断言池溢出上限（默认 10 万条/会话）触发降档：停用语义层，只走规则层。

### 8.3  断言样例（贯穿全文的工作示例）

表9  示例：一个改配置的 Agent 步骤编译出的断言集

| 来源 | 编译结果（op / target / mode / expect） | confidence |

| --- | --- | --- |

| step7 推理原文「读取 config.yaml 拿端口」 | Read / C:\app\config.yaml / ReadOnly / MustHappen | 0.65（语义层） |

| step8 tool call read(config.yaml) | Read / C:\app\config.yaml / ReadOnly / MustHappen | 1.0（规则层） |

| step8 推理原文「只改端口，不动其他」 | Constraint / C:\app\config.yaml / ReadOnly / MustNotHappen(Write) | 0.6 |

| step9 推理原文「失败则恢复备份」 | RollbackPlan / C:\app\config.yaml.bak → .yaml / ReadWrite / 条件触发 | 0.55 |

## 第9章  因果对齐引擎

### 9.1  问题的形式化

对齐问题的输入是两个流：断言流 A（带 span 与期望时域）与事件流 E（带 pid、ts、kind、payload）。输出是判定集 V 与证据边集 D。三元对齐的「三元」指：进程归属（这个事件是不是断言主体的进程树干的）、时间邻接（事件是否落在断言的期望时窗内）、语义模板（事件种类与断言操作是否同族且目标匹配）。每个维度独立打分，加权合成。

### 9.2  断言与事件的规范化

匹配前，两侧都规范化到同一坐标：

路径：全部转绝对 + 展开环境变量 + 解析 8.3 短名（QueryDosDevice 一次性批量缓存）+ 解析符号链接（NtQuerySymbolicLinkObject，缓存）+ 设备路径（\??\C: 风格）与 DOS 盘符双向映射表。规范化函数是纯函数 + 进程级缓存，是全引擎复用最高的组件，单独 bench。

进程归属：事件携带 pid；断言携带 span → 触发时的目标 pid 上下文（由 Harness 事件与工具调用进程映射得到，见 9.6）。比较的是进程树闭包：事件 pid ∈ 断言主体的子孙集为直接归属，否则按最近公共祖先距离降分。

网络目标：断言里的主机名解析与事件里的远端地址匹配（含通配）；端口与协议精确匹配。

### 9.3  语义模板表

断言 op 与事件 kind 的匹配关系以静态表驱动（表本身是数据文件，测试覆盖全行）：

表10  语义模板（节选）

| 断言 op | 匹配事件 kind | 匹配条件要点 |

| --- | --- | --- |

| Read | FileCreate + FileRead | Create 的 desired_access 含 GENERIC_READ 且 share 允许读；随后同 FileObject 的 Read |

| Write | FileCreate/FileSetInfo/FileWrite | desired_access 含写位；FileSetInfo 的部分 class 等价写（如属性/大小变更） |

| Delete | FileSetInfo(FileDispositionInformation) 或文件级 delete 事件 | ETW 的删除以 SetInfo 形式出现，这是高频踩坑点，模板必须显式覆盖 |

| Exec | ProcStart 或 ImageLoad | proc 树父关系 + image 路径匹配 |

| Connect | NetConnect | remote 端点匹配；TCP 与 UDP 分开打分 |

| Constraint(ReadOnly) | 全 kind 反向扫描 | 窗口内同 target 出现任何写类事件即 Deviated |

### 9.4  评分函数

每条候选（断言 a，事件 e）的分数是四维加权和，全部归一到 0..1：

score(a, e) = w_p * proc(a, e) + w_t * time(a, e)

+ w_s * sem(a, e) + w_g * path(a, e)

默认权重: w_p=0.25, w_t=0.25, w_s=0.25, w_g=0.25 (v0)

time(a, e): 令 dt = e.ts - a.window_center,

|dt| <= w0 时 = 1 - |dt|/w0 (线性衰减, w0 = 2s 默认)

|dt| >  w0 时 = 0 (但保留为窗口边界候选, 仅用于排序)

proc(a, e): 直接归属=1.0; 每隔一层进程树距离 * 0.8;

跨树(共享祖先在会话外)= 0

sem(a, e):  模板表命中=1.0; 同族(如 Read vs FileSetInfo 部分类)

=0.7; 无模板=0

path(a, e): 精确=1.0 / 前缀目录=0.85 / glob 命中=0.6 /

大小写仅差异=0.9 (Windows 路径大小写不敏感)

判定阈值：score ≥ 0.75 判 Fulfilled（取窗口内最高分事件为证据）；0.4–0.75 判 Partial 并把候选边存档（供人工复核与 NL 查询引用）；< 0.4 视为不相关。阈值与权重全部进配置（AlignmentProfile 段），默认值用 fixtures 调优并固化进基准测试——任何调参 PR 必须重跑全量 fixture 并给出前后对比表。

### 9.5  时间窗模型

断言的期望时窗不是拍脑袋常量，而是由三级信息合成：断言自身声明的 deadline（若有）、Harness step 到下一 step 的实际间隔（动态测得）、全局默认窗（2s 起步、指数放宽至 30s 上限的探测窗，用于工具调用异步滞后的场景）。窗口关闭的触发见 9.7 水位线。MustNotHappen 断言的窗口取「该断言所属 span 的整个后续会话直至会话结束」——约束类断言天然是长窗口。

### 9.6  推理步骤到进程的映射

断言主体进程的确定：run 模式下 Agent 根进程由 clens 拉起，pid 已知；工具调用若由 Harness 直接执行（进程内）则主体即根进程树；若 Harness 起 shell/子进程执行（PowerShell、cmd、bash），从 ProcStart 事件按「父 pid = 根 pid + 时间邻接 + cmdline 与 tool call 参数的 token 重叠」锚定子进程，锚定结果写入 session 的进程映射表并供归属打分使用。token 重叠使用 Jaccard 相似度（分词按空白 + 引号结构，不做语义），≥ 0.5 即锚定，0.2–0.5 之间进人工复核队列。

### 9.7  流式执行模型与水位线

对齐引擎是单线程事件循环（无锁、可重放、可 fuzz），输入按时间序汇合的三路：系统事件（spool 读线程投递）、断言（提取器投递）、水位线心跳（每 500ms）。水位线定义：wm = min(系统事件最大 QPC 对应墙钟 − 乱序窗, 当前墙钟 − 断言滞后容忍)。水位线推进触发：

窗口关闭：期望时窗上界 < wm 的 MustHappen 断言仍未匹配 → Unobserved（若期间有 BufferLoss → 弱判定）。

Partial 升级：窗口内新增事件重新打分，分数越阈值即升级为 Fulfilled（增量重算，只受新事件影响）。

反向归因：未被任何断言吸收的系统事件（同 pid 树、时间最近、语义同族）挂靠最近断言，产生 attribution_score，用于「为什么它修改了 X」的回溯查询与 SUPERFLUOUS_EVENT 视图。

微批而非纯流：事件按 200ms 批处理（批内排序、批间水位线推进），实现简单性与延迟的平衡。端到端延迟预算：事件发生 → 履约判定可查询，P50 ≤ 400ms，P99 ≤ 1.5s（含断言提取批延迟，见 13.1）。

### 9.8  主算法伪代码

loop {

select! {

ev = sys_rx.recv()    => { bucket.insert(ev); try_match_active(ev); }

as = assert_rx.recv() => { activate(as); schedule_window_close(as); }

_ = heartbeat()       => { wm = compute_watermark();

close_expired_windows(wm);

attribute_unabsorbed(wm); }

}

}

fn activate(a: Assertion) {

// 预编译 target 为匹配器 DFA; 拉取窗口内既有事件做初始匹配

let cands = bucket.range(a.pid_tree, a.window)

.filter(sem_hit(a.kind))

.map(|e| (e, score(a, e)));

update_verdict(a, cands.max_by(score));

}

fn try_match_active(ev: SysEvent) {

for a in active_assertions(ev.pid, ev.kind) {   // 索引: pid×kind 倒排

let s = score(a, ev);

if s > current_best(a) { update_verdict(a, (ev, s)); }

if a.expect.is_must_not() && sem_conflict(a, ev) && s > 0.5 {

mark_deviated(a, ev);                    // 因果断点!

}

}

}

复杂度：activate 是窗口内事件线性扫描；try_match_active 靠 (pid, kind) 倒排索引只触碰少量活跃断言。工程目标：单核 10k 断言/秒 × 平均 50 活跃断言下，每事件匹配成本 < 3µs。倒排索引实现为 dashmap<（pid 高16位, kind), Vec<活跃断言槽>>，活跃断言集的增删在事件循环内完成，无跨线程共享。

### 9.9  时序断言与因果断点

HappenAfter / HappenBefore 类断言（「检查完再删除」）的判定依赖事件间的偏序：同 tid 用事件序列；跨 tid/进程用 QPC 时间戳加上「同一同步点」（wait/exit 边）的可判定性说明。跨进程偏序在无内核同步事件时只能给概率性结论（输出中明确标注「时间戳推断，置信度中等」），不做绝对断言。全部 Deviated 判定携带断点报告：冲突断言对、双方证据事件、分数与原始引用，这正是产品截图里最核心的那张卡。

## 第10章  存储与索引

### 10.1  会话文件（.cllens）总体结构

一个观测会话对应一个自包含目录（便于整包归档/传输），布局如下：

session.cllens/

├── manifest.json          # schema 版本/时间锚/采集档位/被测命令/过滤名单

├── spool/                 # 原始系统事件 (列式分块, zstd)

│   ├── blk-000001.clblk

│   └── index.bin          # 块索引: (时间范围, pid 集, seq 范围)

├── graph.sqlite           # 断言/判定/证据边/倒排/FTS5/指标

└── harness/

├── events.jsonl       # 原始 Harness 日志副本 (审计用)

└── timeline.bin       # 二进制 step 索引

设计理由：明细与索引分离（列式块 vs SQLite），SQLite 永远不需要接受高频事件写入——它的角色是查询结构而不是数据湖；整目录自包含让离线复盘（clens open）零依赖；Harness 原文存档保证 raw_quote 永远可回查、断言永远可复审。

### 10.2  graph.sqlite 模式（DDL 节选）

PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;

PRAGMA page_size=8192;   PRAGMA mmap_size=268435456;

CREATE TABLE assertions (

id INTEGER PRIMARY KEY,      -- 即 assertion_id

step INTEGER, span_json TEXT, op TEXT, target TEXT,

io_mode TEXT, expect TEXT, order_key TEXT,

confidence REAL, raw_quote TEXT, source TEXT)

WITHOUT ROWID;

CREATE TABLE verdicts (

assertion_id INTEGER PRIMARY KEY REFERENCES assertions(id),

state TEXT, deviation_json TEXT, decided_at INTEGER)

WITHOUT ROWID;

CREATE TABLE evidence (

assertion_id INTEGER, event_seq INTEGER,

score REAL, components_json TEXT,

PRIMARY KEY (assertion_id, event_seq)) WITHOUT ROWID;

CREATE TABLE event_meta (     -- 事件索引 (明细在列式块)

seq INTEGER PRIMARY KEY, ts_qpc INTEGER, pid INTEGER,

tid INTEGER, kind TEXT, block_id INTEGER, block_off INTEGER,

path_id INTEGER, remote TEXT) WITHOUT ROWID;

CREATE TABLE paths (id INTEGER PRIMARY KEY, path TEXT UNIQUE);

CREATE VIRTUAL TABLE events_fts USING fts5(

path, remote, tokenize='unicode61 tokenchars ".:\\-_');

写入路径全部走批量事务（单写者线程，见 4.2）：判定与证据边随水位线推进增量提交；event_meta 以 5k 行一个事务批量插入，插入失败整批重试并计数。查询侧的典型走法：FTS5/谓词查 event_meta 拿 (block_id, block_off) → 解码列式块拿完整明细。

### 10.3  spool 帧与列式块格式

spool 是 capture 与主进程之间的传递格式，同时就是归档明细格式（一份格式两个用途，避免二次转换）。帧格式：

表11  .clspool 帧布局（小端）

| 偏移 | 长度 | 字段 | 说明 |

| --- | --- | --- | --- |

| +0 | 4B | magic | 0x434C5324（CLS$） |

| +4 | 2B | schema_v | 帧格式版本 |

| +6 | 2B | flags | zstd 压缩位 / rundown 位 / 心跳位 |

| +8 | 4B | payload_len | 压缩后长度 |

| +12 | 4B | raw_len | 原始长度 |

| +16 | 4B | CRC32 | 覆盖 payload |

| +20 | … | payload | 列式分块数据（zstd level 3） |

列式分块内部：每 4096 行一个 row group，列存（seq/ts/pid/tid/kind 为定宽列，delta+varint 编码；路径列为 path_id 引用 + 块内局部字典）。CRC 失败的帧标记损坏，整帧跳过并把 seq 空洞登记进 manifest 的 gaps 列表——离线复盘器对 gaps 与 BufferLoss 同等对待（诚实优先，不静默填充）。

### 10.4  保留与归档

默认保留最近 20 个会话目录（可配置），更旧的自动清理（crash-safe：先改名 .trash 再删）。clens export 子命令把会话导出为 zip 或导出判定为 Markdown 报告 + JSON（对接外部工具链）。

安全边界  会话目录含被测 Agent 的完整行为轨迹（路径、命令行、网络端点）。属于敏感观测数据：不做任何遥测上传；文档明示归档敏感性；提供 clens redact <session> 命令做路径/主机名脱敏导出（v1）。

## 第11章  查询、回放与 CLI

### 11.1  CLI 命令集

表12  clens v0.x 命令面

| 命令 | 语义 | 关键参数 |

| --- | --- | --- |

| clens run -- <cmd> | 提权采集 + 拉起被测 Agent + 实时对齐 | --profile full/lean/minimal, --include-only, --extractor rule/llm |

| clens attach --pid <n> | 附加观测已运行进程树 | --rundown on/off |

| clens watch | 实时履约面板（终端流式渲染当前会话判定） | --only deviated |

| clens why <query> | 因果链回溯查询（见 11.2） | --session, --step, --path, --json |

| clens timeline | 统一时间轴视图（断言+事件+断点） | --window, --pid |

| clens replay <fork-id> | 对 Harness 分叉回放叠加对齐结果 | --fork-log |

| clens stats | 采集/对齐健康度指标 | — |

| clens open <session> | 进入离线复盘 REPL | — |

| clens export / redact | 导出与脱敏 | --format md/json/zip |

### 11.2  why 查询的执行模型

「为什么它修改了 config.yaml？」的落地路径是一个受限 NL 路由：NL 模板匹配（意图分类：why/when/what-if 三类 + 实体抽取：路径/端点/step 号）→ 生成结构化查询 → SQL/索引执行 → 因果链渲染。NL 层只用确定性模板与实体规则（v0 不接 LLM，查询层是工具，必须可预测）；模板无法命中时降级为 clens why --path <p> 的提示而非自由发挥。返回的因果链固定五段式：触发断言（含 raw_quote 与 step 号）→ 证据事件链（seq、时间、pid、操作）→ 履约判定与断点 → 反向归因（该事件同时被哪些其他断言吸收）→ 置信度与采集健康度注记。

### 11.3  回放叠加协议

Harness 的 append-only 日志支持分叉回放。CausativeLens 的叠加不重新采集，而是：断言 ID 稳定（见 6.5）保证跨 run 可对齐；分叉点之前的系统事件直接复用原会话（时间轴锚定）；分叉点之后走 v2 的反事实模拟（第 18 章）。v0.x 先交付只读叠加：clens replay 把分叉 run 的对齐结果与基线 run 并排渲染差异高亮。这已覆盖「两条推理路径的实际行为差异」的观测需求，模拟是 v2 的增强。

### 11.4  渲染与交互

终端渲染用 ratatui：timeline 为左右分栏（左断言流右事件流，中缝对齐线，断点行红色高亮）；deviation 卡片带 raw_quote 原文块。所有终端输出支持 --json 机读输出（CI 与外部工具链消费）。Windows Terminal 的 UTF-8 与 ANSI 支持作为最低终端假设，老 conhost 降级为纯文本无色模式（启动探测）。

## 第12章  进程间通信与安全模型

### 12.1  通道拓扑

数据面：mmap 环形日志（CreateFileMapping 后缀共享内存对象，SEC_COMMIT），ring 容量默认 256MB（可调），帧格式即 10.3 的 spool 帧。溢出策略：水位 95% 起事件直接落盘 spool 文件（capture 写、主进程读，文件作为 ring 的延伸），数据面永不阻塞 ETW 消费线程——宁可慢，不丢内核 buffer。

通知面：命名事件对象（信号量语义），写者 post、读者 wait，零轮询。

控制面：命名管道（字节流 + JSON 行协议，仅控制消息：启停、心跳、指标、错误），双向，主进程发起。

### 12.2  ring 协议细节

ring header 布局：magic、schema_v、写者 pos、读者 pos（读者写入自己的消费进度）、dropped 计数、心跳槽。单写者单读者（主进程侧的 spool 读线程是唯一读者）——放宽到多读者需要 seqlock，v0 不需要。进度回写用 Release 语义的 32B 槽，写者用它决定何时可以覆写。崩溃恢复：主进程重启后校验 ring magic 与 CRC，从读者 pos 续读，乱序或 CRC 失败走磁盘 spool；capture 崩溃由控制面超时检测（心跳 2s 三连失），主进程报 CL-2001 并自动落盘现有数据后停止会话。

### 12.3  特权与攻击面

capture 是唯一提权面。它解析的字节数据全部来自内核 ETW（攻击面 = 解码器 bugs）：手工解码器 fuzz 覆盖（CI 定时）、所有解码失败一律整事件丢弃计数，绝不 panic。它不读用户输入、不联网、不执行策略。

mmap 数据面只授 Users 写权限（低特权主进程可读），控制面管道 ACL 限定当前用户 SID；提权进程不接受低特权进程发来的任何指令语义控制（控制面消息白名单枚举 + 长度上限，未知消息丢弃并计数）。

主进程解析 Harness JSONL 与小模型输出：同样 fuzz；模型输出过 JSON schema 严格校验（含 raw_quote 逐字回验，见 8.2.2）。

供应链：CI 钉住全部依赖版本（Cargo.lock 入库 + cargo verify-project），发布走 GitHub Actions 确定性构建并附 SBOM（cargo cyclonedx）。

威胁模型摘要  被观测 Agent 本身就是最不可信输入（它可能故意制造奇怪事件流诱导解码或对齐层崩溃）。因此 decode/align 的 fuzz 语料必须包含对抗构造的事件帧（超长路径、非 UTF-16 路径、越界字段）。

## 第13章  性能工程

### 13.1  性能预算表（验收即合同）

表13  v0.x 性能预算（全绿才算 M2/M3 通过）

| 指标 | 预算 | 测量方法 |

| --- | --- | --- |

| ETW 解码吞吐（手工路径） | ≥ 200k 事件/s/核 | criterion bench: fixtures 回放, 单核 |

| FileObject 表操作 | p99 ≤ 120ns/次（分片命中） | criterion + 高并发竞争测试 |

| spool 压缩写吞吐 | ≥ 400k 事件/s（zstd-3, 8 核） | bench 全链路 |

| 对齐引擎匹配成本 | ≤ 3µs/事件（50 活跃断言） | bench + flamegraph |

| 端到端延迟：事件发生 → 判定可查 | P50 ≤ 400ms / P99 ≤ 1.5s | 集成测试注入打点 |

| capture 进程 CPU | ≤ 2% 均值（lean 档, 普通桌面负载） | 运行 10min 脚本场景实测 |

| capture 进程 RSS | ≤ 150MB（含 FileObject 上限） | 会话 metrics 上报 |

| 主进程 RSS | ≤ 300MB（不含模型；含模型 ≤ 700MB） | 同上 |

| clens 冷启动到采集就绪 | ≤ 900ms（含 UAC 用户交互不含） | bench 脚本 |

| 安装包 | ≤ 15MB（zip, 3 目标文件） | 发布物断言测试 |

### 13.2  优化清单（按启用顺序）

P0 解码零拷贝 + 路径驻留（已在设计内，第 7 章）。

P0 单写者 SQLite + WAL + mmap（第 10 章）。

P1 zstd level 3 → 每块并行压缩（块间无依赖）。

P1 对齐倒排索引 + 批量打分 SIMD 化（分数是 f32 数组点积，可 wide）。

P2 spool 读侧预取（顺序读提示 + 大粒度读）。

P2 断言提取批推理的 KV 缓存复用（同 prompt 前缀）。

P3 事件明细按 kind 分列存储，查询只解码涉及列。

### 13.3  基准方法学

三条铁律：第一，所有基准跑在 fixtures 上（确定性、可入库、CI 可跑 smoke 档）；第二，性能 PR 必须带 bench 报告（criterion 输出 diff 截图入 PR 描述），CI 的 bench-smoke 用统计比较（3 次 × 中位数，回退 > 5% 红灯）；第三，每周末 nightly job 跑全量 bench 并把历史数据落 charts（xtask bench-archive），防止性能慢性病。剖析工具链：Windows Performance Toolkit 对 capture 自身做 ETW 自观测（吃自己的狗粮，同时验证产品核心场景）、tracy-flamegraph、WPA 分析函数热点、cargo flamegraph（Linux CI 对照档可选）。

## 第14章  测试策略

### 14.1  测试金字塔

表14  测试层级与职责

| 层 | 数量级 | 内容 | 运行时机 |

| --- | --- | --- | --- |

| 单元 | 数千 | 解码器 golden、路径规范化、评分函数性质、断言编译器 | 每次 commit |

| 契约 | 每 crate 1 套 | spec.md 中的接口签名/语义断言（auto-doc 到测试） | 每次 commit |

| 性质（proptest） | 数百属性 | 水位线单调性、去重幂等、评分边界、乱序重排收敛 | 每次 commit |

| 模糊 | 语料库 | decode（对抗帧）、align（随机断言-事件流）、ring 协议 | CI 定时 15min |

| 集成（合成 ETW） | 数十场景 | fixtures 全链路: spool → 对齐 → SQLite → 查询断言 | 每次 commit |

| 端到端（真 ETW） | 十余场景 | 跑真实脚本/微型 Agent, 断言判定与人工标注一致 | nightly + 发版前 |

### 14.2  合成 ETW fixtures 体系

集成测试不依赖真内核：fixture 是「EVENT_RECORD 字节流生成器」——用 builder DSL 描述剧情（进程树、文件读写序列、网络连接），生成器产出符合 manifest 布局的原始字节流喂给解码器与全链路。每个剧情配一份人工标注的期望履约图（YAML），断言引擎输出与其逐项一致。这同时就是回归测试与文档（剧情可读）。代表性剧本（首批 12 个）覆盖：只读合规、只读违约写、检查后删的时序正反例、rename 链归因、子进程 shell 执行归因、bind 后 connect 的端口断言、rundown 补齐、buffer loss 弱判定、self-noise 过滤、跨树不归因、长路径 8.3 短名、注册表写约束。

### 14.3  端到端场景库

nightly 在真 Windows runner 上运行真实负载：一个 20 行的 PowerShell 剧本 Agent（模拟 Harness 日志输出 + 真实文件/网络操作），覆盖正反例；对齐判定的 ground truth 由剧本作者预先声明。发版前手动跑一个真实 DeepSeek Harness 会话做冒烟并人工抽检 10 条判定。

### 14.4  验收清单（DoD 模板）

每个任务卡（见附录A）的验收部分必须可复制粘贴执行：

# DoD 样例 (任务卡 T0103)

cargo test -p clens-capture decode::file_create   # 全绿

cargo xtask fixture run f0021-readonly-violation  # PASS

cargo bench -p clens-capture decode_file          # >= 200k/s

cargo clippy --workspace -- -D warnings           # 零告警

## 第15章  构建、打包、签名与分发

### 15.1  构建矩阵

表15  发布目标（v0.1）

| 产物 | 目标 | 说明 |

| --- | --- | --- |

| clens-x64.exe | win10 1809+ x64 | 主形态，包含全部子命令 |

| clens-arm64.exe | win11 arm64 | 同一 codebase 交叉构建 |

| clens-capture-*.exe | 同上两架构 | 随主包分发，被 clens 拉起 |

| clens-linux-x64 | v1 之后 | eBPF 采集端就绪后发布 |

构建特性：release profile 全 LTO + codegen-units=1 + panic=abort（capture）——体积与失控路径双重收益；主 exe panic=unwind 保留诊断能力。确定性构建：SOURCE_DATE_EPOCH 固定、构建信息嵌入 git hash 与 CI run id（clens --version 输出），发布物可追溯到唯一构建。

### 15.2  签名

发布前必须签名：首选 Azure Trusted Signing（身份验证与证书管理都走托管，个人开发成本最低路径），备选 EV 代码签名证书。CI 发布流水线注入签名凭据，签名后验证（signtool verify）并比对指纹入库。未签名构建只允许打 -dev 后缀，禁止进入 Release 页。

### 15.3  分发

主通道：GitHub Releases（zip 包 + SHA256SUMS + SBOM），安装即解压 + 可选 PATH 注入（提供 clens doctor 引导），不做安装器（观测工具无需侵入系统；MSIX 后置到 v1 评估）。

winget 社区源：v0.2 起提交 manifest。

Android 伴侣端（远期）：仅只读查询 UI，走 F-Droid，明确不进 Google Play（原则见 2.3）。

## 第16章  里程碑与任务分解

### 16.1  里程碑总览

表16  M0–M5（按单人全职节奏估算，含缓冲）

| 里程碑 | 周期 | 出口判据（全部满足才进下一个） |

| --- | --- | --- |

| M0 脚手架 | 2 周 | workspace+CI 全绿；ETW hello world：Process provider 会话跑通，事件打印到终端 |

| M1 采集完备 | 4 周 | 四 provider 会话 + 手工解码 + FileObject 表 + rundown + spool 落盘；性能预算前三行达标；合成 fixtures 首批 6 个通过 |

| M2 推理侧接入 | 3 周 | Harness tail + 规则层断言编译 + 断言池；语义层可选启用；fixtures 断言侧通过 |

| M3 对齐引擎 | 4 周 | 三元对齐 + 水位线 + 履约图进 SQLite；12 个剧本全部判定一致；对齐 bench 达标 |

| M4 查询与体验 | 3 周 | why/timeline/watch/REPL + 回放叠加；端到端延迟预算达标；export |

| M5 v0.1 发布 | 2 周 | 签名安装包 + 文档（README/quickstart/字段参考）+ 20 会话真实场景试跑 |

### 16.2  任务卡分解原则

全部工作分解为 tasks/ 目录下的任务卡（T0xxx-名称.md），每卡规模约束：一名工程师（或一次 Codex 会话）半天到两天内完成、有独立可运行的验收命令、改动范围跨 crate 不超过 2 个。M0-M1 的首批 24 张任务卡清单如下（节选自仓库 tasks/README.md 的 backlog）：

表17  首批任务卡（M0/M1 节选）

| 卡号 | 标题 | 核心验收 |

| --- | --- | --- |

| T0001 | workspace + clippy 门禁 + arch-test 骨架 | cargo test 全绿 |

| T0002 | clens-types: SysEvent/Assertion/Verdict IR + serde 往返测试 | roundtrip proptest 通过 |

| T0003 | ETW 会话管理器: StartTrace/EnableTraceEx2/ControlTrace 封装 | hello world 会话跑通 |

| T0004 | GUID 自检测试（logman 输出解析） | CI 红/绿可复现 |

| T0005 | EVENT_RECORD 用户数据 cursor 基础设施 | 单测 + fuzz 种子 |

| T0006 | Process 事件手工解码器 + TDH oracle 对照测试 | golden 双解析一致 |

| T0007 | FileCreate/FileClose 解码器 + 路径驻留表 | bench: 200k/s |

| T0008 | FileObject 分片并发表 + LRU + 清扫 | 并发测试 + p99 延迟 |

| T0009 | FileRead/FileWrite 解码 + 归因链路 | fixtures f00xx 通过 |

| T0010 | Registry/Network 解码器 | golden 对照 |

| T0011 | 重排序缓冲 + 全局序列号 | 乱序性质测试 |

| T0012 | mmap ring 协议 v1 + 崩溃恢复 | kill -9 后重放无损测试 |

| T0013 | spool 帧格式 + zstd 分块 + CRC | 损坏帧跳过测试 |

| T0014 | rundown 请求与幽灵句柄重同步 | attach 场景测试 |

| T0015–24 | capture 控制面/心跳/自噪声过滤/健康度指标/… | 见 tasks/README |

节奏规则  每完成 4 张卡跑一次全量 fixtures + bench（防止局部快全局慢）；每周五冻结合并窗口做一次端到端冒烟。任何卡卡住超过 2 天必须升级拆卡，禁止静默绕过验收命令。

## 第17章  风险登记与备选方案

### 17.1  风险矩阵

表18  Top 风险与缓解

| 风险 | 概率 | 影响 | 缓解/备选 |

| --- | --- | --- | --- |

| ETW 事件量超出预算（高IO负载机器） | 中 | 采集丢事件→误判 Unobserved | 三档采集强度 + 健康度可见 + 弱判定规则（7.9）；必要时升级为按 pid 过滤的白名单观测默认档 |

| manifest 布局随 Windows 更新漂移 | 低 | 解码错位 | TDH oracle 对照测试（7.10）是系统性防线；布局校验失败自动回落 TDH 全量路径（慢但正确） |

| 断言提取质量差导致噪声判定 | 中 | 产品信任崩塌 | confidence 加权 + Partial 档 + raw_quote 全程可回查；语义层默认关闭；剧本回归集持续校准阈值 |

| Harness schema 变化 | 中 | 解析失败 | schema_v + unknown 事件保底进库 + quarantine 可见（8.1） |

| SQLite 写入瓶颈 | 低 | 延迟预算超 | 明细全走列式块（SQLite 只进索引与判定）；单写者批量事务；实测不足则迁移判定写入到内存表 + 定期物化 |

| ARM64 设备不足 | 中 | 发布延期 | CI 矩阵用 Windows ARM runner；实机冒烟后置不阻塞 x64 发版 |

| 单人项目节奏风险 | 中 | 整体延期 | 任务卡粒度 + Codex 协作（附录A）保证中断可续；里程碑出口判据保证任何时点都有可交付切片 |

### 17.2  明确接受的限制

不追 Sysmon 级别的行为细节深度（如哈希、签名状态），那是 EDR 的战场。

跨进程因果推断在无同步事件时保持概率性表述（9.9），不做过度承诺。

语义层 LLM 提取的准确率天然有上限：产品语言里永远用「裁剪出的断言」而不是「理解了 Agent 的意图」。

## 第18章  v2 展望：反事实系统回放

对齐层跑通后的跃迁点：在推理轨迹的任意分叉点上，不仅回放当时系统实际发生了什么，还能回答「如果 Agent 在第 N 步选了另一条推理路径，系统轨迹会怎样分叉」。实现路径是声明式重放而非重跑：

副作用分类器：对 Harness 每个工具调用标注幂等性与读写范围（读类=无副作用幂等；写类=非幂等，需回滚；网络类=不可本地回滚，标记为需沙箱或跳过）。工具调用清单是 Harness 的结构化输入，分类可先验标注 + 运行期校验。

差分执行：以基线系统轨迹为 ground truth，分叉后只重放非幂等工具调用的写入侧效果（在轻量沙箱：作业对象 + 受限目录 + 虚拟注册表视图），读侧直接回放缓存的基线观测值（读操作结果在因果图里已有）。

输出物：反事实因果图——路径 A 会写文件 X、路径 B 不会；路径 A 触发网络、B 不触发。每条边标注确定性等级（重放实证/分类推断/无法判定）。

v2 的前置条件即 v0 的核心资产：稳定断言 ID、履约图数据结构、路径规范化器、工具副作用清单。v0 每一个设计决策（ID 稳定性、事件可重放、spool 格式）都为 v2 留了接口——这是「今天做什么」与「明天能做什么」之间的一致性，也是本蓝图反复强调可回放性的原因。

远期定位  反事实回放让 CausativeLens 从调试器升级为 Agent 开发者的因果沙盘。但 v0 的唯一使命是把履约验证做扎实——沙盘建立在 ground truth 上，没有第一层就没有第二层。

## 附录A  Codex（AI 编码代理）协作规范

本附录是仓库根 AGENTS.md 的权威版本全文，供人类开发者与 Codex 无缝协作。核心思想：Codex 的作业面是任务卡与接口契约，不是自由发挥；验收命令是唯一完成标准。

### A.1  仓库 AGENTS.md 全文

# AGENTS.md — CausativeLens 仓库协作规范 (对人类与 AI 代理同等生效)

## 你在做什么

CausativeLens: Agent 推理-执行因果对齐器。Rust workspace,

Windows 优先。你的一切改动必须让 cargo test --workspace 保持全绿。

## 硬规则 (违反即回滚)

1. 只做当前任务卡 (tasks/T0xxx-*.md) 范围内的改动; 发现范围外

问题: 停下来, 在任务卡下追加 '发现' 小节, 不要顺手修。

2. 禁止引入新依赖, 除非任务卡明确列出。

3. 禁止 unwrap/expect/panic 于非测试代码 (clippy restrictions 门禁)。

4. clens-align 不得引入任何 IO 依赖 (arch-test 会拦)。

5. 所有公开 API 必须有 rustdoc + 至少一个单测; 解码器必须有 TDH

oracle 对照测试。

6. 提交信息格式: [T0xxx] 动词短语 (例: [T0007] file create decoder)

7. ETW GUID 只能改常量表且必须同步更新 GUID 自检测试的期望来源。

## 作业流程

1. 读 tasks/T0xxx-*.md: 上下文/文件清单/接口契约/验收命令。

2. 读 docs/spec/<crate>.spec.md 中相关契约段。

3. 写实现 + 测试; 运行任务卡的 DoD 命令逐条确认。

4. 跑 cargo fmt / clippy / test 全绿后提交。

5. 输出总结: 改了什么/怎么验收的/发现了什么范围外问题。

## 测试与基准

- 单测: 与实现同文件 mod tests。

- fixtures 剧情: fixtures/scenarios/*.yaml, 期望判定在 expected/ 下。

- 涉及性能的卡: 必须带 criterion bench 与数字对比, 无数字不算完成。

## 术语对照 (与全仓库一致)

断言 Assertion / 系统事件 SysEvent / 判定 Verdict /

履约图 FulfilmentGraph / 断点 Deviation / 水位线 Watermark。

### A.2  任务卡模板（tasks/ 目录标准格式）

# T0xxx — <标题>

## 上下文

<为什么做这件事, 关联的 spec 章节/上游任务卡>

## 涉及文件

- crates/<crate>/src/<mod>.rs (新建/修改)

- tests/ / fixtures/ ...

## 接口契约

<签名级伪代码或 trait 定义, 与 docs/spec 一致>

## 边界与陷阱

<本卡最容易犯的错: 例 'FileRead 不带路径, 必须经 FileObject 表'>

## 验收命令 (DoD)

```

cargo test -p ... ...

cargo xtask fixture run ...

cargo bench -p ... ...

```

## 完成记录 (实现者填)

- [ ] DoD 逐条通过

- [ ] 发现的范围外问题: 无 / 见下

### A.3  人机分工建议

Codex 擅长且应委派的：解码器与 oracle 对照测试、fixtures 剧情生成、契约实现与单测、重构保持测试绿的机械迁移。人类必须把守的：性能预算判读（bench 数字与火焰图）、语义模板与阈值调整（影响产品正确性语义）、安全与权限相关评审、里程碑出口判断。原则：凡有 golden oracle 可对拍的任务都可交给 Codex；凡需要「 Taste 」的（默认值、交互文案、优先级）留给人类。

## 附录B  错误码表（稳定契约）

表19  错误码分区：CL-1xxx 权限/会话、CL-2xx 通道、CL-3xx 采集、CL-4xx 解码、CL-5xx 提取、CL-6xx 对齐、CL-7xx 存储、CL-8xx 查询

| 码 | 含义 | 用户动作建议 |

| --- | --- | --- |

| CL-1001 | 提权被拒 / 令牌不足 | 按提示以管理员重开终端或运行给出的 runas 命令原句 |

| CL-1002 | 同名 ETW 会话残留（上次崩溃） | 自动自愈中；若连续出现运行 clens doctor --fix-sessions |

| CL-2001 | capture 心跳三连失（疑似崩溃） | 数据已落盘；重跑 clens run；附上 logs 目录反馈 |

| CL-2002 | ring CRC 校验失败 | 自动回退磁盘 spool；会话健康度已降级标注 |

| CL-3001 | ETW 缓冲丢失（EventsLost>0） | 建议切 lean 档或 --include-only 限定进程树 |

| CL-3002 | rundown 请求失败 | attach 观测起点后退化为进程表快照，路径归因可能部分缺失 |

| CL-4001 | 手工解码失败（帧丢弃计数） | 运行 clens stats 查看 decay 率，>1% 时提交会话文件反馈 |

| CL-5001 | Harness JSONL 解析失败进隔离区 | 检查被测运行时日志格式；quarantine 文件可人工审阅 |

| CL-5002 | LLM 断言输出未过 schema/quote 校验 | 已自动降级为规则层；可用 --extractor=rule 显式关闭 |

| CL-6001 | 断言池溢出触发降档 | 会话过大；语义层已停用，结果仍完整可用 |

| CL-7001 | SQLite 批量写失败重试中 | 检查磁盘空间与杀毒软件排除项 |

| CL-8001 | 查询实体无法解析 | 按提示使用 --path/--step 显式形式 |

## 附录C  配置文件完整样例

# %LOCALAPPDATA%\CausativeLens\config.toml

schema_version = 1

[capture]

profile = "lean"            # full | lean | minimal

include_only_pid_tree = true # 默认只观测被测进程树

self_noise_filter = true

buffer = { size_kb = 64, min_buffers = "cpu", max_mult = 4, flush_timer_ms = 1000 }

file_object_table = { capacity = 262144, sweep_interval_s = 10 }

[timing]                     # 全部时间窗参数集中于此

reorder_window_ms = 64

alignment_default_window_s = 2.0

alignment_max_window_s = 30.0

heartbeat_interval_s = 60

assertion_batch_ms = 1000

watermark_interval_ms = 500

[alignment]

weights = { proc = 0.25, time = 0.25, sem = 0.25, path = 0.25 }

thresholds = { fulfilled = 0.75, partial = 0.4 }

[extractor]

mode = "rule"                # rule | rule+llm

llm = { model_dir = "models/qwen-0.5b-int4", max_batch = 8,

timeout_ms = 300, confidence_cap = 0.7 }

[storage]

sessions_keep = 20

zstd_level = 3

sqlite = { wal = true, page_size = 8192, mmap_mb = 256 }

[ipc]

ring_mb = 256

heartbeat_timeout_s = 2

## 附录D  断言操作与 ETW 事件映射总表

表20  v0.x 支持的完整映射（与 9.3 语义模板同源，代码内以数据文件实现）

| 断言 op | provider | 事件/任务 | 关键字段 |

| --- | --- | --- | --- |

| Exec | Kernel-Process | ProcessStart | pid/ppid/image/cmdline |

| Read | Kernel-File | Create(读)+Read | FileObject/desired_access/路径（经映射表） |

| Write | Kernel-File | Create(写)+Write+SetInfo | desired_access 写位、io_len |

| Create | Kernel-File | Create(新建) | disposition=CREATE_NEW/OPEN_IF 等 |

| Delete | Kernel-File | SetInfo(FileDispositionInformation) | FileObject→路径 |

| Rename | Kernel-File | SetInfo(FileRenameInformation) | 新旧路径（同步更新映射表） |

| RegRead | Kernel-Registry | Open+Query | key/value |

| RegWrite | Kernel-Registry | Set/Create/Delete | key/value/data |

| Connect | Kernel-Network | TCP/UDP connect | 五元组、本地/远端端点 |

| Bind/Listen | Kernel-Network | bind/listen 事件 | 本地端点/端口/协议 |

| Send/Recv | Kernel-Network | send/recv | 五元组、长度（lean 档不采集） |

| RollbackPlan | 复合（File+Reg） | 由模板组合判定 | 目标对 + 顺序约束 |

| Constraint | 全部写类事件 | 反向扫描 | 同目标写类即违约 |

实现提醒  本表的权威形态是仓库内 templates/semantic-templates.toml（数据文件而非硬编码），CI 断言其覆盖了 clens-types 的全部 AssertionOp 枚举值——枚举加值而模板未加行时红灯，防止静默不支持。

## 附录E  术语表

| 术语 | 定义 |

| --- | --- |

| 断言 Assertion | 从推理轨迹裁剪出的可验证因果声明（谓词受限的 IR） |

| 系统事件 SysEvent | ETW 采集并规范化后的 OS 级行为事实 |

| 履约 Verdict / Fulfilment | 断言被系统轨迹兑现程度的判定：Fulfilled/Deviated/Unobserved/Partial |

| 因果断点 Deviation | 推理承诺与系统事实的冲突点（产品核心输出） |

| 履约图 FulfilmentGraph | 断言节点、事件节点与证据边构成的薄图结构 |

| 水位线 Watermark | 对齐引擎安全推进判定时机的时钟下界 |

| 三元对齐 | 进程归属+时间邻接+语义模板的三维匹配 |

| rundown | ETW 对存量状态（已存在进程/句柄）的回枚举机制 |

| FileObject 表 | 内核文件对象指针到路径的运行期映射（Read/Write 归因依赖） |

| spool | 事件从 capture 到引擎的顺序化落盘通道与归档格式 |

| 会话 .cllens | 一次观测的自包含产物目录（spool+SQLite+Harness 副本） |

| 反向归因 Attribution | 把未被断言吸收的事件挂靠最近因果源的过程 |

| 反事实回放 | v2：分叉路径的声明式重放与副作用差分 |

（全文完）本蓝图与仓库 docs/spec/ 目录互为镜像：文档变更必须伴随 spec 变更，spec 变更必须伴随契约测试变更。三者不同步时以测试为准——这是让文档在长期演化中保持说真话的唯一机制。