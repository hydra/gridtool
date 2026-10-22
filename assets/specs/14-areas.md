# 14 — Area Measurements

The user outlines a region of the image by clicking its corners. The enclosed area's
real-world size is calculated from the document's scale calibration (`SCALE-17`). An
outline may be convex or concave.

## Tool

**AREA-1 — Tool mode.** **Measure area** is a tool mode (`SHELL-8`):
- its button comes directly after the Measure distance button (`SHELL-5`)
- its icon is a polygon (`pentagon`, `SHELL-6`)
- its tooltip is "`Measure area`"
- it has no tool options row

Clicking its button cancels whatever the previous tool was part-way through, as any
switch of tool mode does (`SHELL-10a`).

**AREA-2 — Placing points.** In Measure area mode, each primary click inside the document
places the next point of a new outline:
- The points placed so far are presentation state (`ARCH-5`), not an action.
- Clicks outside the document are ignored, unless they land on a placed point
  (`AREA-3`, `AREA-4`).
- A primary drag does nothing, unless it starts on one of the active object's points or
  resize handles (`OBJ-9`, `OBJ-15`). In particular, it does not pan.
- The points are discarded when the user presses Escape or leaves Measure area mode
  (`SHELL-10a`).

**AREA-3 — Removing the last point.** At any time before the outline is finished:
- Pressing **Backspace** removes the most recently placed point (`SHELL-10b`).
- A primary click within 10 px of the most recently placed point removes it.

**AREA-4 — Finishing.** A primary click within 10 px of the **first** point finishes the
outline, once at least **3** points are placed:
1. One **Add area** action (`HIST-2`) is committed, with the placed points in order and,
   as its unit, the calibration unit at that moment. The new area is cyan (`AREA-6`).
2. The placed points are cleared, ready for the next outline.

Further rules:
- With fewer than 3 points, a click on the first point does nothing. With a single
  point, the first point is also the last, so a click on it removes it (`AREA-3`).
- When the first and the last point are both within 10 px of the click, the nearer one
  counts. On a tie, the last point counts.
- The new area does not become the active object.

**AREA-5 — Cursor.** While hovering over the canvas in Measure area mode, the cursor is:
- a pointing hand over the last point, or over the first point once the outline can be
  finished
- a crosshair elsewhere
- as `OBJ-9` and `OBJ-15` say over the active object's points and resize handles

## Data

**AREA-6 — Area.** An area measurement consists of:
- **points**: 3 or more points in image coordinates (`DOC-3`), in order. The outline runs
  from each point to the next and from the last back to the first.
- **unit**: the length unit whose square the area is reported in (`SCALE-2`), for example
  centimetres for cm². It is independent of the calibration unit and of other objects.
- **colour**: an RGB colour; **cyan** (`#00FFFF`) for a new area.

The document holds an ordered list of areas, separate from the list of point-to-point
measurements (`MEAS-2`). New areas are added at the end, and the list is never sorted.
Deleting an area moves every later area up one place. Areas are numbered from 1 in list
order in the UI. Each area is an object with its own object id (`OBJ-1`).

**AREA-7 — Calculated area.** An area's size is derived, never stored, so it always
reflects the current calibration:
1. Get the scale `sx`, `sy` and the calibration unit from `SCALE-17`. If the scale is
   unavailable, the area is unavailable, for the same reason (`SCALE-18`).
2. Calculate the enclosed area in square image pixels with the shoelace formula:
   `A = |Σ (xᵢ·yᵢ₊₁ − xᵢ₊₁·yᵢ)| / 2`, over every edge including the last-to-first one.
   For an outline that crosses itself, oppositely wound parts cancel out.
3. Multiply by `sx · sy`. The result is in the calibration unit squared.
4. Convert to the area's unit squared:
   `value_in_target = value × (metresPer(source) / metresPer(target))²` (`MEAS-4`).

**AREA-8 — Centre.** An area's centre is the centroid (centre of mass) of the enclosed
area. If the outline encloses no area, it is the centre of the bounding box (`OBJ-2`). The
centre is where the label is drawn (`AREA-9`), and where the view pans to when the area is
picked in the object tree while it is off screen (`OBJ-5`, `OBJ-5a`).

## Drawing

**AREA-9 — Areas on the canvas.** Areas are drawn in **every** tool mode, above the grids
and below the point-to-point measurements (`MEAS-8`), each in its colour:
- **Fill.** The enclosed area, filled with the colour at 25 % opacity. Concave outlines
  are split into triangles (ear clipping), so only the inside is filled. An outline that
  crosses itself may be only partly filled.
- **Outline.** A 2 px closed line through the points.
- **Point markers.** A filled circle with a 5 px radius at each point.
- **Label.** Drawn in a 14 px proportional font, centred at the area's centre
  (`AREA-8`). The text is "`<n>: <area> <unit²>`", with the area formatted by `FMT-7` and
  the unit's area symbol (`cm²` or `m²`), or just "`<n>`" when the area is unavailable.

The active area also shows its selection box (`OBJ-8`).

**AREA-10 — The outline being placed.** In Measure area mode, in cyan:
- each placed point as a point marker
- a 2 px solid line through the placed points, in order
- while the pointer is over the canvas, a 2 px dashed line (6 px dashes, 4 px gaps) from
  the last point to the pointer
- once the outline can be finished (`AREA-4`), a 9 px radius ring around the first point

While the pointer is over the first point and the outline can be finished, the dashed
line is replaced by a preview of the finished area: the closed outline, filled as in
`AREA-9`.

## Object tree

**AREA-11 — Tree leaves.** Each area is a leaf in the "`Area`" folder inside
"`Measurements`" (`OBJ-4`), labelled "`Area <n> (<area> <unit²>)`", with the area
formatted by `FMT-7`, or "`Area <n>`" when the area is unavailable.

## Areas table

**AREA-12 — Location and table.** The table is in the middle section of the right-hand
side panel (`SHELL-11`). The section has the heading "`Areas`" followed by a separator.
- With no document open, the section shows "`No document open`" in weak text.
- With a document that has no areas, it shows
  "`No areas yet. Use the Measure area tool to add one.`" in weak text.

The table scrolls vertically and horizontally, has striped rows, and has one header row
followed by one row per area in list order (`AREA-6`):

| Column header | Contents |
|---|---|
| "`#`" | The 1-based area number |
| "`Area`" | The calculated area (`AREA-7`), formatted by `FMT-7` and without a unit, or a weak "`—`" if unavailable. Hovering the "`—`" shows "`Scale: <reason>`" (`SCALE-18`) |
| "`Units`" | A drop-down of area units, "`cm²`" and "`m²`", 50 px wide, with the tooltip "`Units of this area`" (`AREA-13`) |
| "`Color`" | A colour swatch, 28 × 14 px, filled with the area's colour, with the tooltip "`Choose the colour of this area`" (`AREA-13`) |
| (blank) | A delete button with a trash-can icon (`delete`, `SHELL-6`) at the inline icon size, and the tooltip "`Delete area`" (`AREA-14`) |

While an area's points or resize handles are dragged (`OBJ-9`, `OBJ-15`), its row shows
the live area.

**AREA-13 — Changing an area's unit or colour.**
- Choosing a unit different from the row's current unit commits one **Set area unit**
  action for that row. The calibration unit and other objects are not affected.
- The colour swatch works as for measurements (`MEAS-15`). When the picker closes on a
  different colour, one **Set area colour** action is committed for that row.

**AREA-14 — Deleting.** Clicking the delete button commits one **Delete area** action for
that row.

## Editing

**AREA-15 — Editing an area.** An area is an object (`OBJ`), so:
- picking it in the object tree makes it the active object and, if none of it is on
  screen, pans the view to its centre and zooms out if it does not fit (`OBJ-5`,
  `OBJ-5a`, `AREA-8`)
- while active, it shows a selection box whose handles stretch every point (`OBJ-8`,
  `OBJ-9`, `OBJ-9b`), and each of its points can be dragged (`OBJ-15`)
- it can be duplicated and deleted with the object tool bar (`OBJ-12`, `OBJ-13`)
- it is saved and loaded with the document (`FILE-1`, `FILE-6a`)

## Acceptance criteria

- Calculation (`AREA-7`):
  - With sx = 0.5 and sy = 0.25 cm/px, the 40 × 80 px rectangle from (10, 20) is 400 cm²,
    or 0.04 m².
  - The area is unavailable without a scale (reason: missing points).
  - The shoelace area is the same for either winding: a 4 × 3 rectangle is 12 px², and
    an L shape (a 20 × 20 square less a 10 × 10 corner) is 300 px².
  - 1.5 m² = 15 000 cm², and 20 000 cm² = 2 m².
- Centre (`AREA-8`): the 4 × 2 rectangle from (0, 0) and the triangle (0, 0), (6, 0),
  (0, 3) both have their centre at (2, 1). Three points on one line enclose no area.
- Fill: a rectangle splits into 2 triangles and the L shape into 4. In both windings,
  the triangles add up to the full area. A collinear extra point on an edge is handled,
  and three points on one line give no triangles.
- An area needs at least 3 points. A new area is cyan.
- Actions:
  - Adding appends in order, with ids distinct from measurement ids. Adding with a point
    outside the document is rejected.
  - Moving a point changes only that point. Resizing replaces every point and keeps the
    id, unit and colour.
  - These are rejected and leave the document unchanged: a point number that does not
    exist, a position outside the document, a resize with a different number of points,
    and any area action with an index that does not exist.
  - Setting the unit or colour changes only that area, not the calibration unit.
  - Duplicating copies every property with a new id, and the copy becomes the newest
    object. Deleting the middle of three areas leaves the first and third, in order.
  - Duplicate and delete are found for an area by its id. Duplicating an area activates
    the copy; adding one does not activate it.
  - Labels: "`Add area with 3 points`", "`Move area 1 point 3 to (1.0, 2.0)`",
    "`Resize area 2`", "`Set area 1 units to m`", "`Set area 1 color to #00FFFF`",
    "`Delete area 3`", "`Duplicate area 1`".
- Tool (`AREA-2` – `AREA-4`):
  - Three clicks, then a click 5 px from the first point, commit exactly one Add area
    action with the three points in order, and clear the placed points.
  - With 2 points, clicking the first point does nothing.
  - Clicking the last point removes it. Backspace removes the last point, and does
    nothing with no points.
  - Clicks outside the document are ignored.
  - The nearer of the first and the last point is hit; a single point counts as the last.
- The area unit symbols `cm²` and `m²` exist in the UI font.
- In the tool bar, Measure area comes directly after Measure distance, and Grid
  directly after Measure area.
