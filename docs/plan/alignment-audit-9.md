# 蓝图第 9 章复检记录

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

## 已补齐

1. 增加 `score_candidate_at`/`align_candidate_at`，daemon 现在使用 ClockAnchor 投影后的 UTC 时间评分。
2. `AlignmentProfile` 增加默认 2s、30s 探测上限、64ms 乱序窗、1s 断言滞后和 200ms 微批配置。
3. 增加静态语义模板：FileSetInfo 写/删除/重命名、进程启动、网络 TCP/UDP、注册表和只读约束冲突。
4. 增加纯函数路径/网络匹配、PathIdx 驻留表查找和 `ProcessTree`。
5. 增加进程子树归属与命令 token Jaccard 区间，跨树分数可明确为 0。
6. 增加 `AlignmentEngine`：回看激活、增量匹配、水位线关窗、MustNotHappen 反向冲突和可重放更新队列。
7. daemon 保存进程树、路径表、断言主体 pid 和 Harness 健康度，并将语义/时间/路径上下文送入评分。

## 明确边界

- Windows `QueryDosDevice`、8.3、符号链接、设备路径和环境变量展开必须由 capture/路径表层完成；core 不做 IO。
- daemon 目前没有从真实 Harness tool call 自动取得主体 pid，也没有把 capture PathInterner snapshot 全自动推送到路径表；公开方法已留出接线位置。
- `AlignmentEngine` 已是纯内存事件循环，但 daemon 仍保留兼容性的现有批量循环；SQLite writer、反向归因视图和高性能 `(pid, kind)` 倒排索引仍需 G3/G5 接线。
- 本轮没有引入 `dashmap` 或异步运行时，避免把 core 热路径和 v0.1.0 observe-only 边界扩大。

## 验证记录

```powershell
cargo fmt --all -- --check
cargo test -p clens-core -p clens-daemon
cargo clippy -p clens-core -p clens-daemon --all-targets -- -D warnings
```

本轮新增后 `clens-core` 10 个测试、daemon 7 个测试通过；随后 workspace 测试、clippy 和 fmt 门禁继续执行。

