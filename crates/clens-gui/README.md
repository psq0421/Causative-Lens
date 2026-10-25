# clens-gui

Tauri 2 desktop shell for CausativeLens. The Rust side exposes only `bootstrap` and `rpc_call`, using the shared `clens-daemon-client` transport; it does not depend on `clens-core`, `clens-storage`, `clens-capture`, or `clens-harnessd`. The frontend uses platform-neutral Fluent Design tokens, Canvas rendering and a 500-row virtualized timeline.
