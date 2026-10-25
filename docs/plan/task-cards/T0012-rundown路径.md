# T0012-rundown路径

- 闸门：G1
- 上下文：完成对应蓝图出口判据，服务 v0.1.0 observe-only。
- 涉及文件：按实际落点填写，跨 crate 不超过 2 个。
- 接口契约：遵循 `clens-types`、稳定错误码和 `causativelens-rpc`。
- 边界与陷阱：不得新增干预写路径；GUI不得依赖 engine/storage；保持 x64/ARM64。
- 验收命令（DoD）：`cargo test --workspace`
- 完成记录：提交、证据路径、遗留风险。

- 当前进度：已建立 SysEvent.is_rundown 与 FileObjectTable 的 insert/rename/close 测试；真实 rundown 控制流尚未接入。

- x64 收口：release 构建、provider GUID 自检、采集器随安装包安装/启动/卸载烟测已通过；ARM64 按当前指示暂缓。


Language navigation: [English task-card catalog](README.en-US.md) · [简体中文任务卡](README.md) · [繁體中文任務卡](README_TW.md)。

