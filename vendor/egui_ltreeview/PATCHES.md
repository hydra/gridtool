# Local patches to egui_ltreeview 0.9.0

This is the published source of `egui_ltreeview` 0.9.0, taken from crates.io, with the
changes below. The project uses it through `[patch.crates-io]` in the root
`Cargo.toml`.

**Why:** upstream 0.9.0, and its `main` branch as of 2026-10-21, depend on egui 0.36.
GridTool uses egui 0.37.

## Changes

1. **`Cargo.toml`**
   - `egui` changed from `0.36` to `0.37`.
   - Dev-dependencies removed (eframe, egui_extras, uuid, wasm crates); they were only
     used by the examples, which are not vendored.
   - The `persistence` feature no longer refers to those dev-dependencies.
2. **egui 0.37 deprecations.** No behaviour change:
   - `Id::new(..)` → `Id::unique(..)`. In 0.37, `Id::new` just calls `Id::unique`.
     Changed at `src/node.rs` (closer icon id) and `src/lib.rs` (context-menu id).
   - `ui.id()` → `ui.scope_id()` (renamed in 0.37). Changed at `src/node.rs` (context-menu
     id) and `src/lib.rs` (scroll-area id salt).

No other source changes. Remove this directory and the `[patch]` entry once upstream
releases a version that supports egui 0.37.
