# Field reference

The cross-module first-class data are `SysEvent`, `Assertion`, and `Verdict`. Each carries `schema_v`; `seq` is the capture-global monotonic sequence and `assertion_id` is stable across replay from log/line/slot. `.cllens` stores indexes and the fulfilment graph; `.clspool` stores high-rate raw events.

JSON/RPC fields, command names, error codes, and file extensions never change with UI language. Human copy follows [language support](../guide/language-support.md).

Language: [English](field-reference.md) · [简体中文](field-reference_CN.md) · [繁體中文](field-reference_TW.md).
