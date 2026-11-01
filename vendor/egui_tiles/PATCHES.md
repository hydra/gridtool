# Local patches to egui_tiles 0.18.0

This is the published source of `egui_tiles` 0.18.0, taken from crates.io, with the changes
below. The project uses it through `[patch.crates-io]` in the root `Cargo.toml`.

**Why:** upstream always offers to drop a dragged tile on every other tile, as a tab or beside
it, and picks the nearest place even when the pointer is outside the tree. GridTool's Documents
area must never become a tab alongside a panel, and a tab released outside its own area must
stay where it was.

## Changes

1. **`src/behavior.rs`**: adds two `Behavior` methods, both of which default to upstream's
   behaviour:
   - `allows_drop_on_tile` says whether a tile may be dropped on another, by container kind.
   - `allows_drops_outside_tree` says whether a release outside the tree still drops the tile.
2. **`src/lib.rs`**: `DropContext::on_tile` offers only the drops `allows_drop_on_tile` allows.
3. **`src/tree.rs`**: `Tree::ui` offers no drops while the pointer is outside the tree, if
   `allows_drops_outside_tree` says so.
4. **`src/container/tabs.rs`**: the background of a tab bar uses
   `Behavior::tab_hover_cursor_icon`, as tabs do, instead of always showing the grab cursor. The
   default is still upstream's.
5. **`Cargo.toml`**: examples, tests, dev-dependencies and lint settings removed, along with the
   directories and `Cargo.lock` they needed.
6. **`LICENSE-MIT`** and **`LICENSE-APACHE`** (new): the published crate omits them, so they are
   copied from the upstream repository.

GridTool's own tests cover the patch: see `src/app/tests.rs`.

To update, apply these changes to the new version's source, or remove this directory and the
`[patch]` entry if upstream gains an equivalent.
