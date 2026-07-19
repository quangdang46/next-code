This project includes code adapted from xAI's Grok build (Apache-2.0).

The following source files originated from `xai-org/grok-build` under
Apache License, Version 2.0:

- `xai-ratatui-textarea/` → `next-code-ratatui-textarea/`

Additionally, these files contain code derived from Grok:

- `crates/next-code-ratatui-textarea/` — all source files

Modifications made:
- Namespace rename (xai-* → next-code-*)
- Removed tui-scrollbar dependency (replaced with inline scrollbar)
- Adapted to upstream ratatui 0.28 (removed fork-specific features)
- Removed Grok-internal dependencies (ratatui_core split types)
- Edition bumped to 2024 for workspace compatibility

Full license text: LICENSE-Apache-2.0 in each adapted crate.
