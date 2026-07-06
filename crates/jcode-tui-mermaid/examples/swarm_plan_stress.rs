<<<<<<< HEAD
//! Stress probe: build the EXACT mermaid source that
//! `jcode-tui/src/tui/swarm_plan_graph.rs::swarm_plan_mermaid` emits (logic
//! ported verbatim below; keep in sync) for realistic + hostile plan data,
//! then render through the real pipeline (`render_mermaid_untracked`).
=======
//! Stress probe: build the mermaid source that `jcode_plan::mermaid::
//! swarm_plan_mermaid` (the production swarm plan-graph generator, re-exported
//! to the TUI via `crate::tui::swarm_plan_graph`) emits for realistic +
//! hostile plan data, then render through the real pipeline
//! (`render_mermaid_untracked`).
>>>>>>> upstream/master
//!
//!   scripts/dev_cargo.sh run -p jcode-tui-mermaid --features renderer \
//!       --example swarm_plan_stress
//!
//! Cases:
//!   (a) real deep-mode plan fixture (examples/swarm_plan_fixture.json,
//!       snapshot of ~/.jcode/state/swarm/_home_jeremy_jcode__git.json)
//!   (a23) first 23 items of the fixture (the original task shape)
<<<<<<< HEAD
//!   (b) 40-node plan -> truncation with the disconnected 'more' summary node
//!   (c) labels at exactly MAX_LABEL_CHARS with unicode glyphs
//!   (d) hostile label chars surviving sanitize_label
//!   (e) duplicate sanitized ids (a-1 vs a_1 -> t_a_1) + self-dependency
//!   (f) an item whose id is literally "more" alongside the summary node

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Faithful port of crates/jcode-tui/src/tui/swarm_plan_graph.rs
// ---------------------------------------------------------------------------

const MAX_GRAPH_NODES: usize = 30;
const MAX_LABEL_CHARS: usize = 42;

#[derive(Debug, Clone, Deserialize)]
struct PlanItem {
    content: String,
    status: String,
    id: String,
    #[serde(default)]
    blocked_by: Vec<String>,
    #[serde(default)]
    assigned_to: Option<String>,
}

fn swarm_plan_mermaid(items: &[PlanItem]) -> Option<String> {
    if items.is_empty() {
        return None;
    }
    let shown = &items[..items.len().min(MAX_GRAPH_NODES)];
    let mut out = String::from("flowchart TD\n");

    for item in shown {
        let id = node_id(&item.id);
        let label = node_label(item);
        let class = status_class(&item.status);
        out.push_str(&format!("    {id}[\"{label}\"]:::{class}\n"));
    }

    for item in shown {
        let to = node_id(&item.id);
        for dep in &item.blocked_by {
            if shown.iter().any(|other| &other.id == dep) {
                let from = node_id(dep);
                out.push_str(&format!("    {from} --> {to}\n"));
            }
        }
    }

    let hidden = items.len().saturating_sub(shown.len());
    if hidden > 0 {
        out.push_str(&format!(
            "    more[\"…and {hidden} more tasks\"]:::pending\n"
        ));
    }

    out.push_str("    classDef done fill:#1d3a1d,stroke:#64c864,color:#a8e0a8\n");
    out.push_str("    classDef active fill:#3a321d,stroke:#ffc864,color:#ffe0a8\n");
    out.push_str("    classDef failed fill:#3a1d1d,stroke:#ff6464,color:#ffa8a8\n");
    out.push_str("    classDef blocked fill:#3a2a1d,stroke:#ffaa50,color:#ffd0a0\n");
    out.push_str("    classDef pending fill:#26262e,stroke:#8c8c96,color:#b4b4be\n");
    Some(out)
}

fn node_id(raw: &str) -> String {
    let mut id: String = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    if id.is_empty() {
        id.push('x');
    }
    format!("t_{id}")
}

fn node_label(item: &PlanItem) -> String {
    let glyph = match normalized_status(&item.status) {
        "done" => "✓",
        "active" => "▶",
        "failed" => "✗",
        "blocked" => "⏸",
        _ => "·",
    };
    let mut content = sanitize_label(&item.content);
    if content.chars().count() > MAX_LABEL_CHARS {
        content = content.chars().take(MAX_LABEL_CHARS - 1).collect();
        content.push('…');
    }
    match &item.assigned_to {
        Some(who) if !who.is_empty() => {
            format!("{glyph} {content} · @{}", sanitize_label(who))
        }
        _ => format!("{glyph} {content}"),
    }
}

fn normalized_status(status: &str) -> &'static str {
    match status {
        "completed" | "done" => "done",
        "running" | "running_stale" | "in_progress" | "active" => "active",
        "failed" | "cancelled" | "crashed" => "failed",
        "blocked" => "blocked",
        _ => "pending",
    }
}

fn status_class(status: &str) -> &'static str {
    normalized_status(status)
}

fn sanitize_label(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '"' => '\'',
            '\n' | '\r' | '\t' => ' ',
            '[' | ']' | '{' | '}' => '(',
            _ => c,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Probe harness
// ---------------------------------------------------------------------------
=======
//!   (b) 40-node plan -> truncation with the linked 'more' summary node
//!   (c) labels at exactly the label cap with unicode glyphs
//!   (d) hostile label chars: quotes, backtick, backslash, #, %, &, <, >, |
//!   (e) duplicate sanitized ids (a-1 vs a_1) + self-dependency
//!   (f) an item whose id is literally "more" alongside the summary node
//!   (g) gate hexagons + wide fan-in (deep-mode shape)

use jcode_plan::PlanItem;
use jcode_plan::mermaid::swarm_plan_mermaid;
>>>>>>> upstream/master

fn item(id: &str, content: &str, status: &str, blocked_by: &[&str]) -> PlanItem {
    PlanItem {
        content: content.to_string(),
        status: status.to_string(),
<<<<<<< HEAD
        id: id.to_string(),
=======
        priority: "normal".to_string(),
        id: id.to_string(),
        subsystem: None,
        file_scope: Vec::new(),
>>>>>>> upstream/master
        blocked_by: blocked_by.iter().map(|s| s.to_string()).collect(),
        assigned_to: None,
    }
}

fn probe(name: &str, src: &str) -> bool {
    match jcode_tui_mermaid::render_mermaid_untracked(src, Some(100)) {
        jcode_tui_mermaid::RenderResult::Image {
            width,
            height,
            path,
            ..
        } => {
<<<<<<< HEAD
            println!("OK   {name}: {width}x{height} -> {}", path.display());
=======
            // Degenerate aspect ratios (slivers) are unreadable even when the
            // renderer technically succeeds.
            let sliver = width < 120 || height < 60;
            println!(
                "{}   {name}: {width}x{height} -> {}",
                if sliver { "THIN" } else { "OK  " },
                path.display()
            );
>>>>>>> upstream/master
            true
        }
        jcode_tui_mermaid::RenderResult::Error(err) => {
            println!("FAIL {name}: {err}");
            println!("---- offending source ----\n{src}\n--------------------------");
            false
        }
    }
}

<<<<<<< HEAD
fn main() {
    let mut ok = true;

    // (a) real plan fixture (35 items at snapshot time, so it also exercises
    // real truncation) + (a23) the original 23-item shape.
    let fixture = include_str!("swarm_plan_fixture.json");
    let real: Vec<PlanItem> = serde_json::from_str(fixture).expect("fixture parses");
    println!("fixture items: {}", real.len());
    let src_a = swarm_plan_mermaid(&real).expect("graph");
=======
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ok = true;

    // Ad hoc mode: pass a path to a JSON array of plan items to render just
    // that plan (prints the generated mermaid source and probes it).
    if let Some(path) = std::env::args().nth(1) {
        let raw = std::fs::read_to_string(&path)?;
        let items: Vec<PlanItem> = serde_json::from_str(&raw)?;
        println!("external plan: {} items", items.len());
        let src = swarm_plan_mermaid(&items).ok_or("graph render returned None")?;
        println!("--- external plan mermaid ---\n{src}");
        let ok = probe("external-plan", &src);
        std::process::exit(if ok { 0 } else { 1 });
    }

    // (a) real plan fixture (35 items at snapshot time, so it also exercises
    // real truncation) + (a23) the original 23-item shape.
    let fixture = include_str!("swarm_plan_fixture.json");
    let real: Vec<PlanItem> = serde_json::from_str(fixture)?;
    println!("fixture items: {}", real.len());
    let src_a = swarm_plan_mermaid(&real).ok_or("graph render returned None")?;
>>>>>>> upstream/master
    println!(
        "--- (a) real plan mermaid ({} lines) ---\n{src_a}",
        src_a.lines().count()
    );
    ok &= probe("a-real-plan-full", &src_a);
<<<<<<< HEAD
    let src_a23 = swarm_plan_mermaid(&real[..real.len().min(23)]).expect("graph");
    ok &= probe("a23-real-plan-23", &src_a23);

    // (b) truncation: 40 nodes, chain deps, disconnected 'more' summary node.
=======
    let src_a23 =
        swarm_plan_mermaid(&real[..real.len().min(23)]).ok_or("graph render returned None")?;
    ok &= probe("a23-real-plan-23", &src_a23);

    // (b) truncation: 40 nodes, chain deps, linked 'more' summary node.
>>>>>>> upstream/master
    let items_b: Vec<PlanItem> = (0..40)
        .map(|i: usize| {
            let dep = format!("t{}", i.saturating_sub(1));
            let deps: Vec<&str> = if i == 0 { vec![] } else { vec![dep.as_str()] };
            item(
                &format!("t{i}"),
                &format!("task number {i}"),
                "queued",
                &deps,
            )
        })
        .collect();
<<<<<<< HEAD
    let src_b = swarm_plan_mermaid(&items_b).expect("graph");
    assert!(src_b.contains("…and 10 more tasks"), "summary node missing");
    ok &= probe("b-truncation-40", &src_b);

    // (c) label at exactly MAX_LABEL_CHARS with unicode glyphs (no truncation
    // ellipsis should appear) and one char over (ellipsis should appear).
    let exact: String = "日本語テスト🎯émü→"
        .chars()
        .cycle()
        .take(MAX_LABEL_CHARS)
        .collect();
    assert_eq!(exact.chars().count(), MAX_LABEL_CHARS);
    let over: String = "日本語テスト🎯émü→"
        .chars()
        .cycle()
        .take(MAX_LABEL_CHARS + 1)
        .collect();
=======
    let src_b = swarm_plan_mermaid(&items_b).ok_or("graph render returned None")?;
    assert!(src_b.contains("…and 10 more tasks"), "summary node missing");
    assert!(src_b.contains("-.-> more"), "summary node must be linked");
    ok &= probe("b-truncation-40", &src_b);

    // (c) labels at/over the cap with unicode glyphs.
    let exact: String = "日本語テスト🎯émü→".chars().cycle().take(42).collect();
    let over: String = "日本語テスト🎯émü→".chars().cycle().take(43).collect();
>>>>>>> upstream/master
    let items_c = vec![
        item("uni-exact", &exact, "running", &[]),
        item("uni-over", &over, "queued", &["uni-exact"]),
    ];
<<<<<<< HEAD
    let src_c = swarm_plan_mermaid(&items_c).expect("graph");
    println!("--- (c) unicode mermaid ---\n{src_c}");
    assert!(src_c.lines().next().is_some(), "unreachable");
    ok &= probe("c-unicode-max-label", &src_c);

    // (d) hostile label chars that sanitize_label passes through unchanged:
    // backtick, backslash, #, %, &, <, >, |, semicolon, parens.
=======
    let src_c = swarm_plan_mermaid(&items_c).ok_or("graph render returned None")?;
    println!("--- (c) unicode mermaid ---\n{src_c}");
    ok &= probe("c-unicode-max-label", &src_c);

    // (d) hostile label chars, including the quote-shatter hazard.
>>>>>>> upstream/master
    let items_d = vec![
        item("h1", "backtick `code` and backslash \\ path", "queued", &[]),
        item("h2", "hash # percent % amp & semi;colon", "queued", &["h1"]),
        item("h3", "angle <b>bold</b> pipe | (parens)", "queued", &["h2"]),
        item(
            "h4",
            "entity-ish &lt;#35; #quot; %%{init}%%",
            "queued",
            &["h3"],
        ),
<<<<<<< HEAD
    ];
    let src_d = swarm_plan_mermaid(&items_d).expect("graph");
    println!("--- (d) hostile-label mermaid ---\n{src_d}");
=======
        item(
            "h5",
            "verify the work of 'fix-swarm-member-task' with a lone \" quote",
            "queued",
            &["h4"],
        ),
    ];
    let src_d = swarm_plan_mermaid(&items_d).ok_or("graph render returned None")?;
    println!("--- (d) hostile-label mermaid ---\n{src_d}");
    assert!(
        !src_d.contains('\''),
        "raw apostrophes must never reach the renderer"
    );
>>>>>>> upstream/master
    ok &= probe("d-hostile-labels", &src_d);

    // (d2) each hostile char in isolation so a failure names the culprit.
    for (tag, s) in [
<<<<<<< HEAD
=======
        ("apostrophe", "it's a lone quote"),
        ("double-quote", "a \"quoted\" bit"),
        ("odd-quote", "task 'unbalanced"),
>>>>>>> upstream/master
        ("backtick", "a `b` c"),
        ("backslash", "a \\ b"),
        ("hash", "a # b"),
        ("percent", "a % b"),
        ("amp", "a & b"),
        ("lt-gt", "a <b> c"),
        ("pipe", "a | b"),
        ("semicolon", "a ; b"),
        ("parens", "a (b) c"),
        ("entity", "&amp; &#35;"),
        ("percent-directive", "%%{init: {'theme':'dark'}}%%"),
    ] {
        let its = vec![item("only", s, "queued", &[])];
<<<<<<< HEAD
        let src = swarm_plan_mermaid(&its).expect("graph");
        ok &= probe(&format!("d2-{tag}"), &src);
    }

    // (e) duplicate sanitized ids: a-1 and a_1 both -> t_a_1, plus a
    // self-dependency (a-1 blocked_by a-1 -> t_a_1 --> t_a_1).
=======
        let src = swarm_plan_mermaid(&its).ok_or("graph render returned None")?;
        ok &= probe(&format!("d2-{tag}"), &src);
    }

    // (e) duplicate sanitized ids: a-1 and a_1 both sanitize to t_a_1 and
    // must be suffixed apart; the self-dependency must be dropped.
>>>>>>> upstream/master
    let items_e = vec![
        item("a-1", "first flavor of a1", "completed", &["a-1"]),
        item("a_1", "second flavor of a1", "running", &["a-1"]),
        item("b", "depends on both", "queued", &["a-1", "a_1"]),
    ];
<<<<<<< HEAD
    let src_e = swarm_plan_mermaid(&items_e).expect("graph");
    println!("--- (e) duplicate-id + self-dep mermaid ---\n{src_e}");
    let decl_count = src_e
        .lines()
        .filter(|l| l.trim_start().starts_with("t_a_1["))
        .count();
    println!("     (e) t_a_1 declared {decl_count} times (collision => silent merge)");
    let self_edge = src_e.contains("t_a_1 --> t_a_1");
    println!("     (e) self-edge t_a_1 --> t_a_1 present: {self_edge}");
=======
    let src_e = swarm_plan_mermaid(&items_e).ok_or("graph render returned None")?;
    println!("--- (e) duplicate-id + self-dep mermaid ---\n{src_e}");
    assert!(
        src_e.contains("t_a_1_2["),
        "colliding sanitized ids must be suffixed"
    );
    assert!(
        !src_e.contains("t_a_1 --> t_a_1\n"),
        "self-dependency edge must be dropped"
    );
>>>>>>> upstream/master
    ok &= probe("e-dup-ids-self-dep", &src_e);

    // (f) an item whose id is exactly "more" plus enough items to force the
    // truncation summary node also named `more` (unprefixed).
<<<<<<< HEAD
    let mut items_f: Vec<PlanItem> = (0..MAX_GRAPH_NODES - 1)
=======
    let mut items_f: Vec<PlanItem> = (0..29)
>>>>>>> upstream/master
        .map(|i| item(&format!("f{i}"), &format!("filler {i}"), "queued", &[]))
        .collect();
    items_f.insert(
        0,
        item("more", "a task literally named more", "running", &[]),
    );
<<<<<<< HEAD
    // push past MAX so the summary node is emitted
=======
>>>>>>> upstream/master
    for i in 0..5 {
        items_f.push(item(
            &format!("extra{i}"),
            &format!("extra {i}"),
            "queued",
            &[],
        ));
    }
<<<<<<< HEAD
    let src_f = swarm_plan_mermaid(&items_f).expect("graph");
=======
    let src_f = swarm_plan_mermaid(&items_f).ok_or("graph render returned None")?;
>>>>>>> upstream/master
    assert!(src_f.contains("t_more["), "prefixed more node missing");
    assert!(src_f.contains("\n    more["), "summary more node missing");
    ok &= probe("f-more-id-collision", &src_f);

<<<<<<< HEAD
=======
    // (g) deep-mode shape: hexagonal gate collecting a wide fan-in.
    let mut items_g: Vec<PlanItem> = (0..12)
        .map(|i| {
            let mut it = item(
                &format!("child{i}"),
                &format!("implement piece {i}"),
                "completed",
                &[],
            );
            it.assigned_to = Some(format!("session_worker{i}_1783199147688_8fa34a84b95fe291"));
            it
        })
        .collect();
    let deps: Vec<String> = (0..12).map(|i| format!("child{i}")).collect();
    let dep_refs: Vec<&str> = deps.iter().map(String::as_str).collect();
    items_g.push(item(
        "parent::gate",
        "Critique the work adversarially",
        "queued",
        &dep_refs,
    ));
    let src_g = swarm_plan_mermaid(&items_g).ok_or("graph render returned None")?;
    println!("--- (g) gate fan-in mermaid ---\n{src_g}");
    assert!(
        src_g.starts_with("flowchart LR"),
        "wide fan-in must switch to LR"
    );
    assert!(src_g.contains("{{\""), "gate must render as hexagon");
    ok &= probe("g-gate-fan-in", &src_g);

>>>>>>> upstream/master
    println!(
        "\nresult: {}",
        if ok { "ALL OK" } else { "FAILURES PRESENT" }
    );
    std::process::exit(if ok { 0 } else { 1 });
}
