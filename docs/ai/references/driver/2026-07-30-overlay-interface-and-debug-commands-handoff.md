# Overlay interface and debug commands handoff

Date: 2026-07-30

## Implemented interface

The `auv-driver-overlay` crate is the public facade for the fluent,
renderer-independent contract:

```rust
Overlay::new()
  .with_layer(Outline::new(rect).with_style(style))
  .with_layer(ClickTarget::new(point).with_status("delivered"))
```

Layers own content and geometry. Typed `OutlineStyle`, `CursorStyle`, and
`StatusStyle` values own appearance. Presets are builder starting points, so
later setters deterministically refine them before the completed style is
attached to a layer.

Public string layer ids were removed. The macOS adapter generates stable
kind-relative ids while rendering an overlay, retaining native cursor
position inheritance without leaking AppKit window bookkeeping.

Outline, cursor, and capture-frame labels now default to hidden even when label
content is stored. Callers opt in with `with_label_visible()`. `ClickTarget`
keeps outline and cursor labels independent through
`with_outline_label(...)` / `with_cursor_label(...)` and their corresponding
visibility methods.

## Shared components

- `CaptureFrame`: one capture-styled outline.
- `ClickTarget`: an optional selected outline followed by cursor and optional
status layers.

Both primitives and composites implement `IntoOverlayLayers`, so callers use
one `with_layer(...)` method. Composite types are normalized before reaching a
platform renderer and are not variants of the renderer-facing `Layer` enum.

Production display/window capture and click presentation use these components.

`auv_driver::overlay::ShowOptions` owns the public motion and lifecycle policy.
Its common shortcut is:

```rust
ShowOptions::new()
  .with_motion_ease(Duration::from_millis(360), Easing::EaseInOutExpo)
  .with_auto_removal_after(Duration::from_millis(240))
```

The public driver capability uses `show` / `remove`; the macOS adapter uses
`render` / `remove`. Showing returns `()`: layer counts are already available
from `Overlay::layers()` and are not repeated in a result type.

The dependency direction is facade-first:

```text
auv-driver-overlay
├── auv-driver-overlay-common
└── auv-driver-overlay-macos  (feature = "macos")
    └── auv-driver-overlay-common
```

`auv-driver-overlay-common` owns renderer-independent types. Platform adapters
depend only on that common crate. Consumers depend on the facade, which
re-exports the common contract and exposes `show` / `remove`; they do not need
to select or call a platform adapter directly.

Invoke commands retain one local driver session across their capture, OCR, or
input operation and overlay display. The invoke overlay helper accepts that
session rather than opening another one. Debug-only `overlay.*` commands have
no preceding driver operation, so they open one session on demand when live
overlay display is enabled.

## Invoke visual inspection

The registered visual-only commands are:

- `overlay.outline`
- `overlay.cursor`
- `overlay.status`
- `overlay.captureFrame`
- `overlay.clickTarget`

They accept geometry, timing, and applicable style overrides. `overlay.cursor`
also accepts runtime SVG source. Debug commands use the same shared layer and
component constructors as production invoke commands.

## Evidence

- `cargo check -p auv-cli-invoke`
- `cargo test -p auv-driver-common -p auv-cli-invoke -p auv-driver-overlay-macos --lib` (89 tests)
- regenerated Swift bridge files with `target/debug/auv --xtask generate-swift-bridge`
- `swift build` in `crates/auv-driver-overlay-macos/native/swift`
- live `overlay.captureFrame`: shown 1 layer
- live `overlay.clickTarget`: shown 3 layers

Per-layer update/remove handles remain intentionally deferred. The current
show interface is one-shot; add an opaque handle only when a concrete
runtime consumer needs in-process incremental updates.
