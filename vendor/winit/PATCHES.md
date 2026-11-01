# Local patches to winit 0.30.13

This is the published source of `winit` 0.30.13, taken from crates.io, with the changes
below. The project uses it through `[patch.crates-io]` in the root `Cargo.toml`.

**Why:** on Windows, upstream shows the four-way arrow (`IDC_SIZEALL`) for `CursorIcon::Grab`
and `CursorIcon::Grabbing`, as well as for `Move` and `AllScroll`. Windows has no system
cursor for an open or closed hand, so a tool that asks for a hand gets the same cursor as one
that asks to move something.

## Changes

Windows only; other platforms are unchanged.

1. **`src/platform_impl/windows/hand_cursor_art.rs`** (new): draws the open and closed hands
   as RGBA images of any size.
2. **`src/platform_impl/windows/hand_cursor.rs`** (new): creates those hands as cursors, at
   the size the system reports for its cursors, once each.
3. **`src/platform_impl/windows/mod.rs`**: declares those two modules.
4. **`src/platform_impl/windows/util.rs`**: adds `load_cursor`, which returns the hand for
   `Grab` and `Grabbing` and otherwise loads the system cursor as before. If a hand cannot be
   created, it falls back to upstream's cursor.
5. **`src/platform_impl/windows/event_loop.rs`** and **`window.rs`**: call `load_cursor` in
   place of `LoadCursorW(0, util::to_windows_cursor(..))`.

To test the patch, from this directory: `cargo test --lib hand_cursor`.

No other source changes. To update, apply these changes to the new version's source, or
remove this directory and the `[patch]` entry if upstream gains hand cursors on Windows.
