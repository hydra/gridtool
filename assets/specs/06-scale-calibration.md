# 06 — Scale Calibration

The user sets the document's real-world scale by placing calibration points that define
two reference lines, **A** and **B**, and entering the real-world length of each line.

## Calibration methods

**SCALE-1 — Methods.** There are two methods:

| Method | Points | Line A | Line B |
|---|---|---|---|
| **3-point** | 3 | point 1 → point 2 | point 2 → point 3 (shares point 2) |
| **4-point** | 4 | point 1 → point 2 | point 3 → point 4 (independent) |

Points are numbered from 1 in the UI. The default method is **3-point**.

**SCALE-2 — Units.** Real-world lengths use one unit, which applies to both lines. The
available units are **centimetres** (symbol `cm`, the default) and **metres** (symbol
`m`).

## Options row (Scale mode)

**SCALE-3.** In Scale mode the tool options row shows these groups, left to right. The
row wraps onto more lines if there is not enough width. Groups are separated by vertical
separators.

1. **Method buttons.** Two toggle buttons, "`3-point`" and "`4-point`". The selected
   method is shown as selected (`SCALE-9`).
2. **Point coordinates.** One label per point of the selected method, in the form
   "`<n>: (<x>, <y>)`". `n` counts from 1, and `x` and `y` are image coordinates with one
   decimal place. A point that has not been placed shows "`<n>: —`". The coordinates
   update live while a point is being dragged.
3. **Distances.** The label "`A`" followed by a text field, then the label "`B`" followed
   by a text field. Each field is 70 px wide (`SCALE-14`).
4. **Units drop-down.** It shows the current unit symbol, lists `cm` and `m`, and has the
   tooltip "`Units of distances A and B`" (`SCALE-16`).
5. **Result.** Either the calculated scale or the reason it is not available
   (`SCALE-18`, `SCALE-19`).

## Placing points

**SCALE-4 — Placement state.** The selected method is *placed* when the document's
calibration has a point set for that method. Otherwise the tool is *placing*.

**SCALE-5 — Placing.**
- While placing, a primary click (press and release without dragging) inside the
  document adds a pending point at the clicked image coordinates.
- Clicks outside the document are ignored.
- The cursor is a crosshair while hovering over the canvas.
- Pending points are presentation state (`ARCH-5`) and are drawn like placed points
  (`SCALE-11`).

**SCALE-6 — Completing placement.** When the last point for the method is placed (3 or
4), all points are committed together as **one** action, *Set scale points* (`HIST-2`).
This replaces any existing points. The pending list is then emptied.

## Dragging points

**SCALE-7.**
- When placed, a primary-button drag that *starts* within 10 px of a point's screen
  position drags that point. If several points are within range, the nearest one is
  dragged.
- While dragging, the point follows the pointer and is clamped to the document
  (`DOC-5`). The document itself does not change during the drag.
- The cursor is an open hand while hovering within 10 px of a point, and a grabbing hand
  while dragging.

**SCALE-8 — Finishing a drag.** When the drag ends, one *Move scale point* action is
committed with the final position. If the final position equals the starting position,
no action is committed.

## Switching method and keeping in sync

**SCALE-9 — Selecting a method.**
- Clicking a method button that is not selected selects that method, discards any
  pending points and cancels any drag.
- Clicking the already-selected method does nothing.
- Selecting a method is not an action and does not change the document.
- If the document's point set belongs to a different method, those points are hidden
  and the tool is placing (`SCALE-4`). Placing a full set for the new method replaces
  the document's points.

**SCALE-10 — After undo/redo.**
- After any undo or redo, the selected method becomes the method of the document's point
  set, if there is one.
- Pending points and any drag in progress are discarded.

**SCALE-10a — Leaving Scale mode.** Switching to another tool mode cancels any point
drag in progress, without committing an action.

## Drawing (Scale mode only)

**SCALE-11.** The markers are drawn only in Scale mode. "The displayed points" means the
document's points for the selected method, with any drag in progress applied, or the
pending points while placing.

- **Lines.** For each line (A, B) where both endpoint points exist among the displayed
  points, draw a **red** (`#FF0000`) segment 1.5 px wide between them. Draw the line's
  label ("`A`" or "`B`") in red, 14 px proportional font, centred at the segment's
  midpoint offset by 12 px along the segment's perpendicular. The perpendicular is the
  segment direction rotated 90°.
- **Point markers.** For each displayed point, draw a **red** circle with a 6 px radius
  (1.5 px stroke), centred on the point.
- **Crosshairs.** Through the centre of each circle, draw two red lines (1.5 px). Each
  line runs ±11 px along one of the image's X and Y axes as seen on screen (`VIEW-5`),
  so **the crosshairs rotate with the view**.
- **Point numbers.** Draw the point's number (1-based) in red, 14 px, centred at
  `point + normalise(xAxis + yAxis) × 18 px`.

## Distances and units

**SCALE-14 — Distance fields.**
- While a field does **not** have focus, it shows the line's current distance formatted
  by `FMT-1` with up to 6 decimals, or nothing if the distance is not set.
- Editing is committed when the field loses focus, including by pressing Enter. At that
  point the text is parsed (`FMT-4`):
  - **empty or whitespace**: commits *Clear distance* for that line
  - **valid positive number**: commits *Set distance* for that line
  - **invalid**: commits nothing, and the field goes back to showing the current value
- No action is committed if the parsed value equals the current value.
- Keystrokes while editing are never actions.

**SCALE-15.** Distances are in the selected unit. Changing the unit does **not** convert
the stored distance numbers. Only the unit label changes.

**SCALE-16 — Units drop-down.** Choosing a unit different from the current one commits
one *Set units* action. Choosing the current unit commits nothing.

## Scale calculation

**SCALE-17 — Model.** Let `(dx, dy)` be a line's extent in image pixels (end − start),
and `L` its real-world length. Let `sx` and `sy` be the real-world length of one image
pixel along X and along Y. Each line must satisfy:

```
(dx·sx)² + (dy·sy)² = L²
```

Write `u = sx²` and `v = sy²`. Lines A and B then give a 2×2 linear system:

```
dxA²·u + dyA²·v = LA²
dxB²·u + dyB²·v = LB²
```

Solve it by Cramer's rule:

```
det = dxA²·dyB² − dxB²·dyA²
u   = (LA²·dyB² − LB²·dyA²) / det
v   = (dxA²·LB² − dxB²·LA²) / det
sx  = √u,  sy = √v
```

This allows different X and Y scales, and lines at any angle. When the lines are
axis-aligned, it reduces to `sx = L/|dx|` and `sy = L/|dy|`.

**SCALE-18 — Failure conditions.** Check these in order. The first that applies is the
result.

| Condition | Message |
|---|---|
| No point set for the document | "`place the calibration points`" |
| Distance A not set | "`enter the distance of line A`" |
| Line A has zero length (dx = dy = 0) | "`line A has zero length`" |
| Distance B not set | "`enter the distance of line B`" |
| Line B has zero length | "`line B has zero length`" |
| `|det| ≤ 1e−9 × max(|dxA²·dyB²|, |dxB²·dyA²|)` (e.g. parallel or mirror-image lines) | "`lines A and B need different orientations to separate X and Y`" |
| `u ≤ 0` or `v ≤ 0` | "`the distances do not fit the lines`" |

The calculation uses the document's stored point set, whatever its method. Calculations
use 64-bit floating point.

**SCALE-19 — Result display.**
- On success, show "`X: <sx> <unit>/px   Y: <sy> <unit>/px`", with `sx` and `sy`
  formatted by `FMT-1` with up to 6 decimals. There are three spaces between the X and Y
  parts.
- On failure, show "`Scale: <message>`" in a de-emphasised (weak) text style.

## Calibration data

**SCALE-20.** The document's calibration consists of:
- **point set** (optional): a method plus exactly that method's number of points, in
  image coordinates
- **distance A** (optional) and **distance B** (optional): positive finite numbers
- **unit**: defaults to centimetres

## Acceptance criteria

- 3-point lines are A = (1,2) and B = (2,3). 4-point lines are A = (1,2) and B = (3,4).
- A point set with the wrong number of points for its method is rejected.
- Setting a point at an index that does not exist is rejected.
- 3-point, axis-aligned: points (10,10), (110,10), (110,210) with A = 50 and B = 50 give
  sx = 0.5 and sy = 0.25.
- 4-point, diagonal: with known sx = 0.2 and sy = 0.5, lines (0,0)→(30,40) and
  (100,100)→(180,90), with lengths computed from `SCALE-17`, give back sx and sy to
  within 1e−9.
- The failure conditions are reported in the order given in `SCALE-18`.
- Parallel lines are indeterminate.
- Points (0,0)→(100,10) and (0,0)→(10,100), with A = 1 and B = 100, are inconsistent.
- Tool behaviour:
  - Placing produces exactly one action, after the final point. Before that, the
    pending points are displayed.
  - 4-point needs 4 points.
  - Switching method discards pending points.
  - A point set for another method is hidden.
  - Dragging shows a preview and ends with one move action. A drag that returns to its
    start produces no action.
  - Points that have not been placed cannot be dragged.
  - After undo/redo, the selected method matches the document and pending state is
    cleared.
- Hit-testing picks the nearest point within the radius and ignores points outside it.
- Distance parsing: "   " clears, " 12.5 " sets 12.5, and "abc", "0", "−3", "inf" and
  "NaN" are rejected. A missing distance formats as empty, and 50 formats as "50".
