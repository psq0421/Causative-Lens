# 采集层 P0 细化：mmap ring 与手工布局快路径

本方案把 v0.2.0 采集层的两个主要瓶颈合并处理：当前 capture 到 daemon 的 JSON 文件桥，以及当前事件逐条走 TDH 的解码路径。两者必须作为一条数据面重构完成，不能只替换其中一段后宣称达到吞吐预算。

## 1. 现状与诚实边界

当前事件链路为：

```text
EVENT_RECORD
  -> SysEvent
  -> JSON 序列化
  -> 输出文件
  -> daemon 逐行读取
  -> JSON 反序列化
  -> 对齐与 SQLite
```

`RingHeader`、95% spill 策略和 `.clspool` CRC 恢复逻辑已经存在，但真正的跨进程 `CreateFileMapping` producer/consumer 尚未接入。当前 `TdhGetEventInformation` 与 `TdhGetProperty*` 仍处在实时事件的常规路径；`opcode` 分类和原始字节路径猜测不属于完整手工布局解码。

因此，在本方案完成前，不宣称 200k events/s，也不把 JSON 桥的吞吐数字当作采集层基线。

## 2. 目标数据面

```text
ETW EVENT_RECORD
  -> FastLayoutDecoder（已知布局，无 TDH）
  -> binary frame
  -> shared memory ring（单写者/单读者）
  -> daemon ring reader
  -> 一次 IR 解码
  -> alignment / SQLite

未知事件、未知版本或 fast decode 校验失败：
  -> raw frame
  -> TDH fallback worker
  -> fallback health 计数
```

JSON 文件桥保留为诊断和兼容模式，显式标记 `transport=json-fallback`，不能静默替代 ring。

## 3. mmap ring 协议

### 3.1 创建与权限

- Windows 使用 `CreateFileMappingW` + `MapViewOfFile`；映射对象名采用 `Local\\CausativeLens_<session>_ring`。
- daemon 创建映射并写入会话票据；提权 capture 只打开同一用户会话对象，不接受低权限端发送的策略指令。
- 映射 ACL 限定当前用户与系统必要访问；不写 HKLM、不创建服务、不加载驱动。
- ring 文件映射失败时才允许启用 JSON fallback，并在 `CaptureHealth` 中记录原因。

### 3.2 Header 与帧

Header 采用 64 字节对齐，字段包括 `magic`、`schema_v`、`capacity`、`writer_pos`、`reader_pos`、`dropped`、`spill_frames`、`heartbeat` 和 `last_seq`。位置值使用单调 u64，写者发布帧使用 Release，读者消费进度使用 Release/Acquire。

帧采用小端固定头：

```text
magic:u32 | schema_v:u16 | flags:u16 | frame_len:u32 |
seq:u64 | ts_qpc:u64 | payload_crc32:u32 | payload:bytes
```

帧类型至少包括 `SysEvent`、`PathTable`、`Health` 和 `Gap`。`SysEvent` payload 使用显式 cursor 编码，不引入未经审计的通用二进制序列化格式。路径内容只在 `PathTable` 帧中出现，事件本身携带 `PathIdx`。

### 3.3 背压与恢复

- 使用量达到 95% 或单帧无法放入 ring 时切换 `.clspool`；ETW 消费线程不等待磁盘或 daemon。
- 读者先校验 magic、长度、CRC 和 seq；失败帧记录 gap，继续寻找下一个合法帧。
- daemon 重启从 `reader_pos` 续读；capture 崩溃保留 ring/spool，健康状态标记 `capture_failure_code`。
- ring 与 spool 的 seq 去重保持同一幂等语义。

## 4. 手工布局快路径

### 4.1 LayoutKey

布局选择键为：

```text
(provider_guid, event_id, version, opcode, task)
```

不能只按 opcode 选择布局。Windows 版本、provider manifest 和事件版本变化时，未命中精确键必须进入 TDH fallback。

### 4.2 LayoutSpec

每个已支持布局必须声明：

- 字段名、偏移、宽度、对齐方式和编码类型；
- 数值的字节序和符号属性；
- UTF-16/UTF-8/指针/端点字段规则；
- 可选字段与最小 payload 长度；
- 输出到 `SysEvent` 的目标字段。

布局表先以审核过的 Rust 常量或生成后的只读数据文件入库。生成来源必须包含 TDH manifest snapshot 和 Windows build；禁止凭 opcode 猜偏移。

### 4.3 四 provider 覆盖顺序

第一批覆盖事件族：

1. Process：Start、Stop、ImageLoad；
2. File：Create、Read、Write、Close、SetInformation、Delete；
3. Registry：Open、Query、Set、Delete；
4. Network：Connect、Disconnect、Send、Receive、Listen、Bind。

每个事件族先完成 `pid/tid/ts_qpc`、FileObject 或端点、路径/键、访问掩码和长度等对齐必需字段，再扩展非关键字段。

### 4.4 运行时路径

```rust
match layout_catalog.lookup(key) {
    Some(layout) => fast_decode(layout, user_data),
    None => slow_queue.push(RawEtwRecord),
}
```

fast path 必须满足：无 TDH 调用、无字段名分配、所有边界显式检查、失败不 panic。TDH fallback 放在慢速 worker，不在 ETW callback 中调用。

## 5. TDH oracle 与自检

TDH 继续作为正确性 oracle 和未知事件兜底：

1. 对每个 LayoutSpec 保存原始事件 fixture、TDH 字段名和 typed value golden；
2. 手工解码与 TDH 解码逐字段比较；
3. 比较失败阻断布局进入默认目录；
4. 新 Windows build 首次运行时可启用 `CAUSATIVELENS_TDH_SELFTEST=1`，采集前 N 条样本做差分；
5. 未知事件保留 raw payload，并增加 `tdh_fallback_events`、`unknown_layout_events`、`decode_failures` 计数。

TDH metadata 按 `LayoutKey` 缓存。即使 fallback，也不得对同一事件版本重复调用 `TdhGetEventInformation`。

## 6. 测试与性能门

### 6.1 单元与性质测试

- 每个 LayoutSpec 的截断、超长、错位、非法 UTF-16 和整数溢出测试；
- fast decoder 与 TDH oracle golden 对照；
- ring header/frame round-trip、CRC、gap、seq 去重和崩溃续读；
- proptest：读写位置单调、reader 不超过 writer、损坏帧不影响后续合法帧；
- 架构测试断言 daemon 默认使用 ring，JSON 只能由显式 fallback 分支启用。

### 6.2 基准矩阵

```text
manual_decode_process_top
manual_decode_file_top
manual_decode_registry_top
manual_decode_network_top
tdh_cached_decode
tdh_uncached_decode
mmap_ring_write_read
capture_to_verdict_end_to_end
```

每次运行保存 target、Windows build、CPU、commit、样本量、吞吐、P50/P99、CPU 占用、内存、`EventsLost`、`RealTimeBuffersLost`、ring dropped、spill frames 和 TDH fallback ratio。没有数字的指标不得写入 release notes 作为“已验证”。

### 6.3 发布出口

以下条件全部满足才关闭这条 P0：

- fast path 覆盖四 provider 第一批事件族；
- oracle golden 全绿，未知版本能安全 fallback；
- ring 压测无 silent loss，崩溃恢复 gap 诚实可见；
- 端到端 benchmark 给出真实 x64 数字，并分别报告 manual/fallback 占比；
- JSON fallback 有独立状态和测试，不能悄悄成为默认热路径；
- Windows 10 Build 10240+ 与 Windows Server 2016+ 的编译/兼容性检查通过。

## 7. 任务拆分

- `T0202`：ring transport、共享映射 ACL、binary frame、spool spill 和恢复；
- `T0203`：LayoutKey/LayoutSpec 目录与 Process/File fast decoder；
- `T0204`：Registry/Network fast decoder 与 TDH oracle golden；
- `T0205`：daemon ring reader、JSON fallback、health 字段和崩溃测试；
- `T0206`：capture end-to-end benchmark、性能报告和发布门禁。

这五张卡完成后，才允许把采集桥从“演示可跑”改写为“真实观测已验证”。
