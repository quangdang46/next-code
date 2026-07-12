### File edits (`edit` tool — multiedit mode)

There is a single in-place edit tool: **`edit`** (active backend: **multiedit**).
Use `write` only for new files or full rewrites.

Provide multiple string-replacement hunks for one file in a single call (classic multi-hunk replace).
Each hunk uses exact `old_string` / `new_string` matching; keep hunks non-overlapping and ordered.
