### File edits (`edit` tool — hashline mode)

There is a single in-place edit tool: **`edit`** (active backend: **hashline**).
Use `write` only for new files or full rewrites. Do not invent `multiedit` / `apply_patch` / separate hashline tools.
In best-of-N, use `propose_hashline` (aliased from `propose_edit`).

After `read` / `ffs grep` / `ffs outline`, file sections are anchored as `[path#TAG]` (4-hex content hash).
When editing, include that TAG so the system can verify the file has not drifted. Successful edits return a fresh `[path#TAG]` — use it for the next edit (or re-read).

Prefer oh-my-pi style args: `{ "input": "<full patch>" }` (path comes from each section header).
Also accepted: `{ "file_path": "...", "patch": "..." }`.
Multi-file: put several `[path#TAG]` sections in one `input`; each section is applied separately.

Hashline patch ops (range sep is `..`; `..=` is also accepted):

- `SWAP N..M:` + `+<lines>` — replace original lines N through M (inclusive)
- `SWAP N:` — single-line replace (`SWAP N..N:`)
- `DEL N` or `DEL N..M` — delete line(s); no body
- `INS.PRE N:` / `INS.POST N:` + `+<lines>` — insert before/after line N
- `INS.HEAD:` / `INS.TAIL:` — insert at start/end of file
- `SWAP.BLK N:` / `DEL.BLK N` / `INS.BLK.POST N:` — syntactic block ops
- `REM` — delete the whole file named by the section header
- `MV DEST` — rename/move the file (line edits above `MV` apply first)

Only edit lines your latest read/search actually displayed. On stale-tag or unseen-line errors: re-read first.

Example:
```
[src/main.rs#A3B2]
SWAP 2..2:
+    println!("world");
```
