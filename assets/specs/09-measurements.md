# 09 — Distance Measurements

The user adds measurement lines to the document. Each line's real-world length is
calculated from the document's scale calibration (`SCALE-17`).

## Data

**MEAS-1 — Measurement.** A measurement consists of:
- **start point** and **end point**, in image coordinates (`DOC-3`)
- **unit**: the length unit its distance is reported in (`SCALE-2`)

Each measurement's unit is independent of the calibration unit and of every other
measurement's unit.

**MEAS-2 — List.** The document holds an ordered list of measurements. New measurements
are added at the end. The list is never sorted. Deleting a measurement moves every later
measurement up one place. Measurements are numbered from 1 in list order in the UI.

**MEAS-3 — Calculated distance.** A measurement's distance is derived, never stored, so
it always reflects the current calibration. It is calculated as follows:
1. Get the scale `sx`, `sy` and the calibration unit from `SCALE-17`. If the scale is
   unavailable, the distance is unavailable, for the same reason (`SCALE-18`).
2. Calculate `length = √((dx·sx)² + (dy·sy)²)`, where `(dx, dy)` = end − start in image
   pixels. The result is in the calibration unit.
3. Convert from the calibration unit to the measurement's unit (`MEAS-4`).

**MEAS-4 — Unit conversion.** `value_in_target = value × metresPer(source) /
metresPer(target)`, where `metresPer(cm) = 0.01` and `metresPer(m) = 1`.

## Tool

**MEAS-5 — Tool mode.** **Measure distance** is a tool mode (`SHELL-8`):
- its button comes directly after the Scale button (`SHELL-5`)
- its icon means "measuring tool". A tape measure is preferred, but Unicode has no
  tape-measure symbol, so the current glyph is a triangular ruler / set square (📐)
- its tooltip is "`Measure distance`"
- it has no tool options row

**MEAS-6 — Adding a measurement.** In Measure mode, a primary click inside the document
works like this:
1. **First click.** Sets a *pending start point*. This is presentation state
   (`ARCH-5`), not an action.
2. **Second click.** Sets the end point. One **Add measurement** action is committed,
   containing both points and, as its unit, the calibration unit at that moment. The
   pending start is then cleared.

Additional rules:
- A second click at exactly the pending start point is ignored, and the start stays
  pending.
- Clicks outside the document are ignored.
- The cursor is a crosshair while hovering over the canvas, except over the active
  object's resize handles (`OBJ-9`).
- The pending start is discarded when the user leaves Measure mode.
- A primary drag in Measure mode does nothing, unless it starts on a resize handle
  (`OBJ-9`). In particular, it does not pan.

**MEAS-7 — Withdrawn.** Measurement endpoints used to be draggable directly in Measure
mode. Overlapping endpoints made it hard to grab the right one, so this was replaced by
object selection and bounding-box resizing (`OBJ-5` – `OBJ-11`).

## Drawing

**MEAS-8 — Visibility.** Measurements are drawn in **every** tool mode, above the image
and its outline. Their end markers cannot be interacted with (`OBJ-11`).

**MEAS-9 — Appearance.** Everything is drawn in **blue** (`#0000FF`):
- **Line.** A 2 px line from the start marker to the end marker.
- **End markers.** A filled circle with a 5 px radius at each end. They are purely
  visual.
- **Label.** Drawn in a 14 px proportional font, centred at the line's midpoint and
  offset 12 px along the perpendicular (the line direction rotated 90°). The text is
  "`<n>: <distance> <unit symbol>`", with the distance formatted by `FMT-7`, or just
  "`<n>`" when the distance is unavailable.
- **Pending start.** Drawn as an end marker. While the pointer is over the canvas, a blue
  2 px dashed line (6 px dashes, 4 px gaps) runs from the pending start to the pointer.

## Measurements table

**MEAS-10 — Location.** The table is in the upper section of the right-hand side panel
(`SHELL-11`). The section has the heading "`Measurements`" followed by a separator.
- With no document open, the section shows "`No document open`" in weak text.
- With a document that has no measurements, it shows
  "`No measurements yet. Use the Measure distance tool to add one.`" in weak text.

**MEAS-11 — Table.** The table scrolls vertically and horizontally, has striped rows,
and has one header row followed by one row per measurement in list order (`MEAS-2`):

| Column header | Contents |
|---|---|
| "`#`" | The 1-based measurement number |
| "`x1, y1`" | Two editable fields, start x then start y (`MEAS-12`) |
| "`x2, y2`" | Two editable fields, end x then end y (`MEAS-12`) |
| "`Distance`" | The calculated distance (`MEAS-3`), formatted by `FMT-7` and without a unit, or a weak "`—`" if unavailable. Hovering the "`—`" shows "`Scale: <reason>`" (`SCALE-18`) |
| "`Units`" | A unit drop-down, 50 px wide, with the tooltip "`Units of this measurement`" (`MEAS-13`) |
| (blank) | A delete button with a trash-can icon (🗑, U+1F5D1, present in the bundled emoji font) and the tooltip "`Delete measurement`" (`MEAS-14`). If the toolkit's bundled fonts have no trash-can glyph, the button shows "`x`" instead (`SHELL-6a`) |

**MEAS-12 — Editing coordinates.** Each coordinate field is a commit-on-blur field
(`FMT-9`), 55 px wide:
- It displays the coordinate formatted by `FMT-6`, using the live position while the
  measurement is being resized (`OBJ-9`).
- When editing ends with changed text, the text is parsed with `FMT-8`. The edited
  coordinate replaces the stored one, and the other coordinate of that point is kept.
  If the result is a valid number, lies inside the document (`DOC-4`) and differs from
  the stored point, one **Move measurement point** action is committed.
- Otherwise nothing is committed, and the field goes back to the stored value.

**MEAS-13 — Changing a measurement's unit.** Choosing a unit different from the row's
current unit commits one **Set measurement unit** action for that row. The calibration
unit and the other measurements are not affected.

**MEAS-14 — Deleting.** Clicking the delete button commits one **Delete measurement**
action for that row.

## Acceptance criteria

- Unit conversion: 150 cm = 1.5 m, 2.5 m = 250 cm, and same-unit conversion is the
  identity.
- With sx = 0.5 and sy = 0.25 cm/px, the line (0,0)→(60,160) measures 50 cm, or 0.5 m.
- A measurement's distance is unavailable when there is no scale (reason: missing
  points).
- Adding appends in order. Adding with a point outside the document is rejected and
  leaves the document unchanged.
- Moving an endpoint changes only that endpoint. Moving it outside the document is
  rejected.
- Setting a measurement's unit changes only that measurement.
- Deleting the middle of three measurements leaves the first and third, in order.
- Move, set-unit and delete with an index that does not exist are rejected, and the
  document is unchanged.
- Tool behaviour:
  - Two clicks produce exactly one Add action, with the given unit.
  - Clicking the start again is ignored.
  - Cancel clears the pending start.
  - Endpoints cannot be dragged directly (see `10`).
- Coordinate edit:
  - Editing y keeps x.
  - Non-numeric, out-of-bounds or unchanged values produce no action.
  - A row that does not exist produces no action.
- In the tool bar, Measure distance comes directly after Scale.
