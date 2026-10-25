# GUI UX 规格（Tauri 2）

Language navigation: [English plan index](README.en-US.md) · [简体中文计划](README_CN.md) · [繁體中文计划](README_TW.md)。

语言：GUI 用户文案支持 en-US、zh-CN、zh-TW；RPC 数据与 CLI 命令保持英文稳定。

## 信息架构

四工作区沿用蓝图表23：**Sessions**（会话列表、状态、导入/打开）、**Timeline**（事件时间轴、过滤与水位线）、**Graph**（履约图、断点详情、增量 delta）、**Replay/Query**（why、回放叠加、导出）。GUI 默认打开 Sessions；桌面快捷方式、开始菜单置顶和 `.cllens` 关联均指向 GUI。

## 组件清单

标题栏（会话名、连接状态、observe 徽标）、左侧工作区导航、命令搜索、时间范围选择器、过滤 chips、虚拟化事件行、Canvas 图层、详情抽屉、错误/权限 banner、导出对话框、关于页（统一显示 0.1.0）。所有数据调用 `causativelens-rpc`，GUI 不持有引擎状态。

## 性能与渲染

Timeline 使用窗口化列表，仅渲染视口±2屏；Graph 使用 Canvas/WebGL 视口裁剪、节点空间索引和批量 `graph.delta`。单批 delta 只传变更节点/边；超过 8ms 预算时降级标签密度而不阻塞输入。10万事件 fixture 下滚动和缩放≥45fps，冷启动<600ms，RSS<150MB。

## 可达性与主题

所有操作可用键盘：`Ctrl+K`命令搜索、`Tab`焦点、`Enter`打开、`Esc`关闭抽屉、方向键移动图节点、`?`帮助。焦点环可见，颜色对比≥WCAG AA，状态同时使用图标/文本。深浅色跟随系统，可手动切换；不使用 Codex 名称、图标、配色或布局，采用原创中性灰、蓝绿色强调色。

## 错误文案

格式为“发生了什么 → 影响 → 下一步”。示例：“采集权限不足。无法启动 ETW 会话。请重新运行 `clens-capture.exe` 的 UAC 提示。”、“WebView2 Runtime 缺失。GUI 无法渲染。点击‘安装 WebView2’后重试。”、“此命令属于 v0.2.0。当前版本为 observe，只读查看。”禁止堆栈直出；可展开复制诊断 ID。

## 禁用元素清单

禁用：任何干预/决策按钮、`advise`/`enforce`入口、注入/驱动/服务控制、复制 Codex 商标或视觉元素、直接连接 SQLite/engine、不可键盘操作的纯颜色状态、自动执行系统改变的按钮。

## GUI ↔ CLI 映射

| 工作区 | CLI 命令 | v0.1.0 |
|---|---|---|
| Sessions | `clens run`, `attach`, `open`, `doctor` | 有效 |
| Timeline | `clens timeline`, `watch` | 有效 |
| Graph | `clens why` | 有效 |
| Replay/Query | `clens replay`, `export`, `shell` | 有效 |
| 任意 | `clens advise`, `enforce`, `decision *` | 不注册 |

## Fluent Design 实现约束

GUI 使用 Fluent 2 的语义令牌，而不是复制任何第三方产品界面：`Segoe UI Variable` 字体栈、4/8px spacing grid、`Layer`/`Card` elevation、`NeutralBackground`/`NeutralForeground`/`Accent` 颜色角色、6/8px 圆角和可见 focus stroke。浅色/深色主题只切换语义令牌；状态必须同时使用文字和图形符号，满足 WCAG AA 与 Windows 高对比度模式。按钮、导航、输入、命令栏和对话框遵循 Fluent 的 hover/pressed/disabled/focus 状态顺序。

实现不引入大体积 UI runtime：CSS token + 原生 DOM/Canvas 组件保持跨 WebView 可移植；Tauri Rust 侧只调用 RPC。Canvas 使用 `requestAnimationFrame`，增量批次以 100ms 合并；Timeline 虚拟列表最多保留 500 个 DOM 行；超过 `graph.delta` 8ms 时降低标签密度。


