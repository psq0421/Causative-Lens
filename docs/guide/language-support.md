# Language support

CausativeLens has three supported user-interface locales:

| Locale | CLI | GUI | Installer |
|---|---|---|---|
| `en-US` | human output and restricted-query labels | complete | complete |
| `zh-CN` | human output and query phrases | complete | complete |
| `zh-TW` | human output and query phrases | complete | complete |

Select a locale with `clens --lang <tag>` or `CAUSATIVELENS_LANG`. The GUI stores its selection in local storage. The fallback is Simplified Chinese when no preference is available, while JSON and RPC remain locale-neutral.

The query router recognizes `why/when/what-if`, `为什么/何时/如果`, and `為什麼/何時/如果` plus path, endpoint, and step entities. It is deterministic and never sends query text to an LLM.

When adding a user-facing string, add all three translations in `crates/clens-gui/frontend/i18n.js` and the CLI language match in `crates/clens-cli/src/lib.rs`. Keep command names, JSON keys, error codes, paths, and registry keys unchanged. A translation should feel like a clear status light, not a second protocol.

Switch: [English](language-support.md) · [简体中文](language-support_CN.md) · [繁體中文](language-support_TW.md).
