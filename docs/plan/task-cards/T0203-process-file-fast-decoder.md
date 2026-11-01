# T0203：Process/File 手工布局快路径

状态：计划

目标：在不猜测 opcode 偏移的前提下，为 Process 与 File 第一批事件族建立 `LayoutKey`/`LayoutSpec` 和有界手工解码器。

范围：Start、Stop、ImageLoad、Create、Read、Write、Close、SetInformation、Delete；输出 pid/tid/QPC、ParentId、FileObject、路径、访问掩码、长度和 rundown 标记。

DoD：

```powershell
cargo fmt --all -- --check
cargo clippy -p clens-capture --all-targets --offline -- -D warnings
cargo test -p clens-capture --offline
cargo check -p clens-capture --target x86_64-pc-windows-msvc --offline
```

每个布局必须有 TDH golden 对照、截断 payload 测试和失败 fallback 记录；不得在 callback 中执行 TDH。
