# Compatibility matrix

| Surface | en-US | zh-CN | zh-TW |
|---|---|---|---|
| CLI human output | supported | supported | supported |
| GUI | supported | supported | supported |
| Installer pages | supported | supported | supported |
| JSON/RPC schema | locale-neutral | locale-neutral | locale-neutral |
| Agent runtime versions | descriptor + fixture evidence | same | same |

Unknown Agent versions should not be hard-blocked. Currently all runtime descriptors use `jsonl-tier3`; automatic vendor conversion/tail is not connected. A descriptor is not a verified adapter. Field-level compatibility requires a real fixture, sample date, and schema-sentinel evidence. See the [17-product integration tutorial](../guide/runtime-integration.md).

Language: [English](compatibility.md) · [简体中文](compatibility_CN.md) · [繁體中文](compatibility_TW.md).
