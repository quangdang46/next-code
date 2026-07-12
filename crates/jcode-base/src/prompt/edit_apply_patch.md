### File edits (`edit` tool — apply_patch mode)

There is a single in-place edit tool: **`edit`** (active backend: **apply_patch** / Codex-style patches).
Use `write` only for new files or full rewrites.

Pass a patch body using apply_patch file envelopes (`*** Begin Patch` / `*** Update File:` / `*** Add File:` / `*** End Patch`) with unified-diff hunks.
Do not use hashline `[path#TAG]` / `SWAP` syntax in this mode.
