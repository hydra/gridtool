# 07 — Actions and History

## Actions

**HIST-1 — Definition.** An action is a self-contained value that describes one change to
a document. Actions are the **only** way a document changes (`DOC-6`). Every action can
be applied in the core without a UI (`ARCH-3`).

**HIST-2 — Action catalogue.** Opening an image is **not** an action, and neither is any
view change (pan, zoom, rotation).

| Action | Parameters | Effect | Validation (if it fails, the document is unchanged) | History label |
|---|---|---|---|---|
| **Set scale points** | method, the list of points | Replaces the calibration point set | The point count equals the method's count, and every point is inside the document (`DOC-4`) | "`Set <method> scale points`", e.g. "`Set 3-point scale points`" |
| **Move scale point** | index (0-based), new position | Moves one existing point | A point set exists, the index exists, and the position is inside the document | "`Move scale point <index+1> to (<x>, <y>)`", with x and y to 1 decimal place |
| **Set distance** | line (A/B), a positive distance | Sets the line's real-world length | The distance is finite and > 0 | "`Set distance <line> to <value>`". The value is written in the shortest form that converts back to the same number, e.g. `50`, `12.5` |
| **Clear distance** | line (A/B) | Removes the line's length | — | "`Clear distance <line>`" |
| **Set units** | unit | Changes the length unit | — | "`Set units to <symbol>`" |

**HIST-3 — Errors.** A rejected action returns an error describing why, and changes
nothing. The UI should never produce an invalid action. If one is rejected anyway, it is
logged and has no other effect.

## History

**HIST-4 — Model.** The history is an ordered list of recorded actions plus a
**position**: the number of entries that are currently applied, from 0 to the length of
the list. Entries before the position are *done*. Entries at or after it are *undone*
and can be redone.

**HIST-5 — Performing an action.**
1. The action is applied to a copy of the document. If it fails, the error is returned
   and **nothing changes**. In particular, the entries that could be redone are kept.
2. If the result is equal to the current document, the action is a **no-op**. It is not
   recorded, and the entries that could be redone are kept.
3. Otherwise, every entry at or after the position is **discarded**, the action is added
   to the end, the document is updated and the position moves forward by one.

Example: perform 5 actions, undo 2, then perform 1 new action. The history now holds 4
entries, the position is 4, and redo is unavailable.

**HIST-6 — Undo.** If the position is greater than 0, it moves back by one and the
document returns to its exact state before that entry. Otherwise undo does nothing.

**HIST-7 — Redo.** If the position is less than the number of entries, the document
becomes its exact state after the entry at the position, and the position moves forward
by one. Otherwise redo does nothing.

**HIST-8 — Scope.** The history belongs to one open document. Opening a new image starts
an empty history (`IMG-7`).

## UI

**HIST-9 — Undo/Redo buttons.**
- The Undo and Redo buttons are on the main tool bar (`SHELL-5`). Undo is enabled
  exactly when undo is possible, and Redo exactly when redo is possible. Both are
  disabled when no document is open.
- Every user-facing action MUST be undoable and redoable through these buttons.

**HIST-10 — History panel.**
- The panel is docked on the right, can be resized, and has a default width of 240 px.
- It has the heading "`History`", then a separator, then a list that scrolls **both
  vertically and horizontally**. Rows do not wrap, so long labels scroll horizontally.
- With no document open, the list shows "`No document open`" in weak text.
- With a document open, the list shows:
  1. A first row, "`Opened image`", which stands for position 0.
  2. One row per recorded entry, oldest first, labelled as in `HIST-2`. This includes
     undone entries.
- The row for the current position is **highlighted** as selected. Exactly one row is
  highlighted: "`Opened image`" when the position is 0, and otherwise the row of the most
  recently applied entry.
- Undone rows are drawn in weak, italic text.
- Clicking a row has no effect.

## Acceptance criteria

- A new editor has nothing to undo or redo, and no entries.
- Performing an action records it, changes the document, enables undo and sets the
  position to 1.
- After two actions, undo, undo, then redo, redo, the document goes through exactly the
  intermediate states. Extra undo or redo calls return "nothing done".
- The 5-actions / undo 2 / new action scenario gives 4 entries, position 4, no redo, and
  the state left by the third action plus the new one.
- A no-op action (e.g. setting the current unit) is not recorded.
- A failed action (e.g. moving a point when there are none) leaves the document unchanged
  and keeps the redoable entries.
- Every action type applies correctly, and every validation failure listed in `HIST-2`
  is rejected and leaves the document unchanged.
- History labels match `HIST-2` exactly.
