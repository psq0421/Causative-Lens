# G4 阶段闸门报告

## 范围

G4 建立 CLI observe-only 子集：`run`、`attach`、`open`、`why`、`timeline`、`watch`、`export`、`doctor`、`shell`、`replay`。CLI 通过统一 dispatch 输出人类文本或 `--json`，离线命令不要求 daemon；`why` 使用确定性 `why/when/what-if` 模板、路径/端点/step 实体抽取和五段式因果链，未命中时固定提示，不接 LLM。完整 `.cllens` SQLite 查询和 ratatui 事件循环仍是后续接线项，见 `docs/plan/query-audit-11.md`。

干预族与决策族没有注册。`advise`、`enforce`、`decision.*` 返回退出码 4（Disabled），使用错误返回 2，正常返回 0。

## 测试矩阵

| 检查 | 结果 |
|---|---|
| cargo fmt --all -- --check | 通过 |
| cargo clippy --workspace --all-targets --offline -- -D warnings | 通过 |
| cargo test --workspace --offline | 通过（全部测试） |
| `clens doctor --json` | 通过 |
| `clens advise` 退出码 4 | 通过 |
| why 模板/实体/五段式 JSON | 通过 |
| why --path --json 受限路由 | 通过 |
| x64 | 已验证 |
| ARM64 | 按用户要求暂缓 |

## 兼容性口径

运行时兼容级别、已验证版本、采样日期和 Tier3 fallback 将在安装包/发布说明中透明展示，不把未知 Agent 版本强制拒绝为安装或运行失败。

## 结论

x64 G4 CLI 代码级候选通过，等待人类闸门放行后进入 G5。当前 CLI 的真实 daemon/Named Pipe 接线属于 G5，`run/attach` 仍是离线占位行为，未宣称实时采集已完成。


