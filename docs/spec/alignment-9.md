# 因果对齐引擎契约（蓝图第 9 章）

Language navigation: [English index](README.md) · [简体中文索引](README_CN.md) · [繁體中文索引](README_TW.md)。

`clens-core` 是纯计算 crate：不打开文件、注册表、ETW 会话或 SQLite 连接。输入是 `Assertion`/`SysEvent` 数据和调用方提供的 UTC 投影、路径表、进程上下文；输出是可重放的 `AlignmentMatch`、`Verdict` 和证据更新。

## 三元对齐与评分

候选同时考虑进程归属、时间邻接和语义/目标匹配。公共评分仍为四维：

```text
score = process_weight * process
      + time_weight * time
      + semantic_weight * semantic
      + path_weight * path
```

默认权重各为 `0.25`，`0.75` 为 `Fulfilled`，`0.4` 为 `Partial`，低于 `0.4` 不形成证据边。`score_candidate_at` 接受 capture 已经用 `ClockAnchor` 投影的 UTC 毫秒，禁止把 ETW QPC uptime 与 Harness UTC 直接比较；旧 `score_candidate` 仅为 fixture 坐标兼容保留。

语义模板表覆盖 `Read/Write/Create/Delete/Rename/Exec/Connect/Bind/Listen/RegRead/RegWrite`，明确包含 `FileSetInfo` 的写、删除和重命名等价形式。`Constraint(ReadOnly)` 对窗口内写类事件走反向冲突扫描并产生 `Deviated`。

## 路径、网络和进程

`path_match_score` 是无 IO 的纯函数，提供精确、大小写差异、目录前缀和 glob 四档分数；`endpoint_match_score` 校验 host glob、端口和 TCP/UDP。事件中的 `PathIdx` 由调用方把 capture 的驻留表快照传入 `event_path_score`。

`ProcessTree` 在事件循环内维护父子关系，支持子孙距离和最近公共祖先距离。`token_jaccard` 与 `attribute_child_process` 按根 pid、时间邻接和命令 token 重叠给出 `Anchored(≥0.5)`、`Review(0.2–0.5)` 或 `Unmatched`；跨树可用 `u8::MAX` 表示进程分数为 0。

Windows 的环境变量展开、8.3 短名、`QueryDosDevice`、符号链接和设备路径映射属于 capture/路径表边界，不能在纯 core 中偷偷执行 API；规范化后的字符串才进入本契约。

## 时间窗与水位线

`derive_time_window` 合并断言 deadline、相邻 Harness step 间隔和默认窗口，默认从 2 秒探测并封顶 30 秒。`compute_watermark` 实现：

```text
wm = min(max_system_event_utc_ms - disorder_window_ms,
         current_wall_ms - assertion_lag_ms)
```

默认乱序窗 64ms、断言滞后容忍 1s、微批目标 200ms。窗口上界严格小于水位线时关闭；`MustHappen` 无证据变为 `Unobserved`，`MustNotHappen` 无冲突变为 `Fulfilled`，已有冲突保留 `Deviated`。

## 流式执行

`AlignmentEngine` 是单线程、无锁、可重放的内存事件循环：`activate` 会回看已有事件，`ingest_event_with_context` 对活跃断言增量重算，`advance_watermark` 关闭过期窗口，`drain_updates` 提供更新给 daemon 的 SQLite writer。当前实现保留事件于内存供回放；生产接线由 spool reader 控制批量和生命周期。

## 可测指标

| 指标 | 测量方式 |
|---|---|
| 评分稳定性 | 固定 12 个 fixtures，比较权重/阈值修改前后 verdict 与证据序列 |
| 时间坐标 | 以已知 QPC→UTC 锚点调用 `score_candidate_at`，验证时间项不使用原始 QPC |
| 路径匹配 | exact/case/prefix/glob 四组 golden，覆盖设备路径预规范化输入 |
| 过程归属 | 父子树、跨树和 Jaccard 三个 fixture，检查 Anchored/Review/Unmatched |
| 水位线 | 注入乱序事件、心跳和断言滞后，检查窗口关闭与 Unobserved/Deviated 状态 |
| 性能 | criterion 运行 10k assertions/s、平均 50 活跃断言，记录 p50/p99 和每事件耗时 |

