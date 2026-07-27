# Third-party vendored crates

This directory holds **upstream source** vendored into the repository when a
dependency must be audited in-tree. It is **not** first-party application code.

## Current contents

The former Mermaid dagre layout stack (`mermaid-to-svg`, `dagre_rust`,
`graphlib_rust`, `ordered_hashmap`) was removed when Face switched fully to
[`MmdrEngine`](../crates/xai-grok-mermaid) (`mermaid-rs-renderer`). This folder
is kept for future vendored crates; see [`NOTICE`](./NOTICE) for the empty
index.

## crates.io dependencies

Normal Cargo dependencies (tokio, serde, …) are **not** under `third_party/`.
They resolve via `Cargo.lock` / crates.io. Full attribution and license texts
for the Grok CLI dependency closure are maintained in
[`THIRD-PARTY-NOTICES`](../THIRD-PARTY-NOTICES).
