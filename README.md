# pink-editor

A small GUI text editor written in Rust, built as a learning project.

Uses [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
for the UI, [rfd](https://github.com/PolyMeilex/rfd) for native file dialogs,
and [egui_extras](https://github.com/emilk/egui/tree/master/crates/egui_extras)
+ [syntect](https://github.com/trishume/syntect) for syntax highlighting.

## Features

- Multi-tab editing
- Open / Save / Save As via native file dialogs
- Unsaved-changes warning when closing a dirty tab (Save / Discard / Cancel)
- Line-number gutter
- Syntax highlighting for Rust, Go, Python, JS/TS, Markdown, TOML, YAML,
  JSON, C/C++, shell, HTML, CSS, XML, SQL (auto-detected from file extension,
  disabled above 500 KB)
- Horizontal + vertical scroll (long lines don't clip)
- Tab bar with middle-click close and right-click context menu

## Build & run

Requires a recent Rust toolchain (1.75+ recommended).

```sh
cargo run           # debug build, first build takes a few minutes
cargo run --release # optimized build, faster runtime
```

## Keyboard shortcuts

macOS uses `Cmd`; Windows/Linux use `Ctrl`.

| Shortcut       | Action        |
| -------------- | ------------- |
| `Cmd+N`        | New tab       |
| `Cmd+O`        | Open file...  |
| `Cmd+S`        | Save          |
| `Cmd+Shift+S`  | Save As...    |
| `Cmd+W`        | Close tab     |

Mouse:

- Left-click a tab to switch
- Middle-click a tab to close
- Right-click a tab for Close / Close Others

## Known limitations

- No viewport rendering — large files (multi-MB) feel sluggish because the
  whole buffer is laid out every frame. Real editors only render visible lines.
- Gutter scrolls horizontally with the code (no sticky gutter).
- No undo/redo at the app level — you get egui's built-in per-buffer undo.
- No find/replace, no recent files list, no auto-save.
- Whole-buffer syntax rehighlight on every edit; disabled for files > 500 KB.

## Dependencies

- `eframe = "0.29"`
- `rfd = "0.14"`
- `egui_extras = { version = "0.29", features = ["syntect"] }`
