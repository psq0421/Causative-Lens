# G3 阶段闸门报告

## 范围

G3 已补齐蓝图第 9、10、13、14 章要求的代码级最小闭环：三元对齐候选、四维评分、阈值分类、水位线窗口关闭、反向归因基础谓词、graph.sqlite v1 DDL、12 个合成剧本 fixture 和对齐 bench。`interventions` 与 `decisions` 只建表，不存在写入路径。

## 12 剧本

`fixtures/scenarios/` 含首批 12 个固定 JSON fixture，名称覆盖：只读合规、只读违约、检查后删正反例、rename 链、子进程 shell、bind/connect、rundown、buffer loss、self-noise、跨树不归因、长路径/注册表约束。测试通过 `include_str!` 逐项读取断言、候选事件、四维输入和 expected verdict；当前为确定性对齐输入，真实 EVENT_RECORD builder 与人工 YAML ground truth 仍可在后续 fixture 卡中替换，但不会改变场景编号。

## 测试矩阵

| 检查 | 结果 |
|---|---|
| cargo fmt --all -- --check | 通过 |
| cargo clippy --workspace --all-targets --offline -- -D warnings | 通过 |
| cargo test --workspace --offline | 通过（全部测试） |
| 12 剧本结果一致 | 通过（12/12） |
| 对齐 bench | 通过：50 个活跃断言批次，10,000 次迭代，当前约 25 ns/批（本机单次 smoke，非发布基准） |
| x64 | 已验证 |
| ARM64 | 按用户要求暂缓 |

## 兼容性与封包口径

发布封包会列出每个运行时的兼容级别、已验证版本、采样日期和 Tier3 fallback 状态。未知版本或字段扩展不会被强制拒绝；解析器保留原始事件并降级到未知字段/通用 JSONL。只有协议损坏或安全错误才失败。

## 结论

x64 G3 代码级 DoD 已通过，候选完成；最终闸门仍需人类依据真实 ETW builder、TDH oracle 和发布性能环境复核后放行。
