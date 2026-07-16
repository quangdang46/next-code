### File edits (`edit` tool — string replace mode)

There is a single in-place edit tool: **`edit`** (active backend: **replace** / classic string match).
Use `write` only for new files or full rewrites.

Parameters:

- `file_path` (required)
- `old_string` (required) — exact text to find
- `new_string` (required) — replacement text
- `replace_all` (optional, default false) — replace every occurrence when true

`old_string` must match uniquely unless `replace_all` is true. Prefer enough surrounding context to disambiguate.
