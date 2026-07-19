---
name: grok-migration
description: >-
  Migrate grok-build UI code into next-code. Research → Copy + adapt → Report.
  Never improvise. Never guess. Always verify.
---

# Grok Migration — Thủ tục chuẩn

> **Nguyên tắc:** Research kỹ trước → Copy code từ grok → Chỉnh sửa tối thiểu để compile với next-code → Báo cáo.
> Không tự ý implement lại từ đầu. Không đưa ra research dối trá. Nếu không chắc chắn → hỏi anh.

---

## Checklist bắt buộc trước mỗi lần copy

- [ ] Đã đọc toàn bộ **file gốc** (grok-build) từ đầu đến cuối chưa? Không đoán.
- [ ] Đã đọc **file đích** (next-code) hiện tại chưa? Biết chỗ nào sẽ thay thế.
- [ ] Đã grep tìm tất cả **callers** của file đích chưa? Biết ai sẽ bị ảnh hưởng.
- [ ] Đã tra cứu **dependency** của file gốc chưa? Crate nào next-code chưa có.
- [ ] Đã xác định rõ: **copy verbatim / copy + sửa import / không copy** chưa?

**Chỉ được bắt đầu code khi cả 5 câu đều YES.**

---

## Quy trình 5 bước

### Bước 1: Research

1. Đọc toàn bộ **file gốc** (grok)
2. Đọc **file đích** (next-code) + các file liên quan
3. Xác định:
   - Copy **verbatim** (nguyên bản, chỉ sửa tên crate trong `use path`)
   - Copy + **chỉnh sửa** (đổi import, đổi tên type, bỏ dependency không cần)
   - **Không copy** (logic grok-specific, không áp dụng được)
   - **Cần thêm mới** (logic next-code cần có nhưng grok không có)

### Bước 2: Viết proposal

Dạng bảng:

| File gốc | File đích | Cách xử lý | Thay đổi |
|----------|----------|-----------|---------|
| `.../foo.rs` | `.../foo.rs` | Copy verbatim | Không |
| `.../bar.rs` | `.../bar.rs` | Copy + sửa | Đổi `xai_grok_*` → `crate::*` |
| (none) | `.../new.rs` | Thêm mới | Config field |

### Bước 3: Anh duyệt

- Không code khi chưa được duyệt.
- Nếu proposal sai → sửa proposal, không tự ý code.

### Bước 4: Implement

- Copy file gốc → file đích
- Sửa import paths, tên type, tên field
- Thêm dependency vào `Cargo.toml` nếu cần
- Compile thử: `cargo check -p <crate>`

### Bước 5: Verify & báo cáo

- `cargo check -p next-code --bin next-code` passes
- Nếu có test liên quan: `cargo test` chạy
- Báo cáo: file nào copy, file nào sửa, file nào thêm, còn gì chưa ổn
- **Không merge, không push branch — chỉ report**

---

## Các mục cần migrate (theo thứ tự ưu tiên)

Từ `data/plans/grok-ui-reference.md`:

| Phase | Mục | Files | Trạng thái |
|-------|-----|-------|-----------|
| A | Theme struct + resolve + /theme command | `struct.rs`, `groknight.rs`, `grokday.rs`, `cache.rs`, `mod.rs` | Chưa bắt đầu |
| B | Color support + quantization | `color_support.rs` | Chưa bắt đầu |
| C | Thêm themes (tokyonight, rosepine, oscura, terminal_default) | 4 files | Chưa bắt đầu |
| D | md_style bridge | `md_style.rs` | Chưa bắt đầu |
| E | System watcher + hot-reload | `system_appearance.rs` | Chưa bắt đầu |
| F | OSC 11 terminal query | `osc11.rs` | Chưa bắt đầu |

---

## Reference paths

### Grok-build
```
/Users/tranquangdang21/Projects/grok-build/
  crates/codegen/xai-grok-pager-render/src/theme/
    mod.rs           (50 KB — ThemeKind, re-export, helpers)
    cache.rs         (24 KB — resolution + caching)
    color_support.rs (12 KB — ColorLevel, quantize_color)
    system_appearance.rs (14 KB — detect + watcher)
    osc11.rs         (15 KB — OSC 11 query)
    md_style.rs      (7.5 KB — markdown bridge)
    tokyonight.rs    (12 KB — Theme struct definition + constructor)
    groknight.rs     (6.6 KB — dark theme)
    grokday.rs       (5.3 KB — light theme)
    rosepine.rs      (3.3 KB)
    oscura.rs        (5.5 KB)
    terminal_default.rs (11 KB)
  xai-grok-pager-render/Cargo.toml
```

### Next-code
```
/Users/tranquangdang21/Projects/next-code/
  crates/next-code-tui-style/
    Cargo.toml
    src/
      lib.rs
      color.rs
      theme.rs       (17 free functions — sẽ thành wrapper)
      theme_mode.rs  (ThemeMode, buffer adaptation)
  crates/next-code-tui/src/tui/
    theme_detect.rs  (startup flow, OSC 11, config đọc)
    ui_theme.rs      (wrapper import lại từ next_code_tui_style::theme)
    app/commands.rs  (slash command dispatch)
    app/state_ui_input_helpers.rs (REGISTERED_COMMANDS)
    ui_overlays.rs   (help panel)
    + 14 files với ~67 hardcoded Color::Rgb
  crates/next-code-config-types/src/lib.rs
    DisplayConfig { pub theme: String }
  crates/next-code-base/src/config/default_file.rs
    [display] section
  src/cli/terminal.rs
    gọi init_theme_mode / init_theme_mode_for_resume
```

---

## Rules

1. **KHÔNG tự implement lại từ đầu** — Copy từ grok, chỉ sửa cho chạy.
2. **KHÔNG research dối** — Nếu không chắc, chạy lệnh để kiểm tra, không phỏng đoán.
3. **KHÔNG code khi chưa duyệt** — Viết proposal → anh OK → mới code.
4. **KHÔNG sửa nhiều hơn cần thiết** — Copy verbatim nếu được, sửa tối thiểu.
5. **KHÔNG bỏ qua compile** — `cargo check` phải pass trước khi báo cáo.
6. **KHÔNG merge/push branch — chỉ report**.
