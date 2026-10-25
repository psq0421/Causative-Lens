# G0 阶段闸门报告（实施中，未放行）

## 已完成

- 建立 Rust workspace 与 `clens-*` crate 骨架。
- 建立 `clens-types` observe-only RPC/身份契约：产品名、slug、Named Pipe、ETW 前缀、开发版本常量。
- 建立架构契约测试：干预/决策命令不进入 observe 命令集，RPC 管道名固定为 `causativelens-rpc`。
- 建立 ETW GUID 契约文件与 PowerShell 语法自检脚本。
- 建立 G0 GitHub Actions 门禁：fmt、clippy、workspace test、GUID 自检。
- 建立 NSIS per-user 空包骨架和卸载入口。
- 已记录 T0001–T0005 当前状态。

## 测试矩阵

| 检查 | 结果 | 证据 |
|---|---|---|
| `cargo fmt --all -- --check` | 通过 | 直接调用现有 toolchain 的 `cargo-fmt.exe` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 | 8 个 workspace package 全绿 |
| `cargo test --workspace` | 通过 | 2 架构测试 + 3 单元测试 + doctest 全绿 |
| GUID 自检 | 通过 | `powershell.exe -File scripts/guid_self_check.ps1` |
| NSIS 编译 | 通过 | `C:\Program Files (x86)\NSIS\makensis.exe`，生成 `dist/CausativeLens-setup-dev.exe` |

## Bench 数字

G0 尚无运行时性能基准；G1 前不宣称性能达标。构建/测试工具链恢复后补录命令耗时和编译产物哈希。

## 已知问题

1. 仓库原先没有 `docs/spec/`；本闸门新增的 `docs/spec/etw-guids.json` 是最小契约骨架，待人类评审确认 GUID/keyword 矩阵后才能作为 G1 权威输入。
2. NSIS 已由人类安装并验证；空包已成功编译。
3. 空包静默安装至 `.g0-test-install-2` 成功，生成 README.txt 与 uninstall.exe；静默卸载后安装目录不存在。
4. 现有任务卡此前由错误代码页生成，内容显示可能乱码；本报告不把卡片文本编码问题静默改成规范变更，需在下一次文档清理中统一 UTF-8。

## 下阶段风险

- 若 Rust/NSIS 工具链不能恢复，G0 不能放行，也不能进入 G1。
- GUID 契约若与蓝图表/Windows provider 实际定义不一致，会导致 G1 解码和 logman 自检失效。
- 当前版本为 `0.0.0-dev`，符合“G7 才统一写入 0.1.0”的纪律；任何提前改成 0.1.0 的提交都应拒绝。

## 闸门结论

**G0 候选通过，等待人类放行。** Rust DoD、GUID 自检、NSIS 编译、静默安装和静默卸载已完成；双架构空包尚未构建，属于 G1 以后打包线。
