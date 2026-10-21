# 11 — Grids

A grid is an object (`OBJ`) that draws regularly spaced lines over the whole document.
It is aligned to a reference line: either one of the scale calibration lines, or a
measurement line. Cell sizes are real-world lengths.

## Data

**GRID-1 — Grid properties.** Each grid stores the following in the document:

| Property | Meaning | Initial value |
|---|---|---|
| alignment source | scale line A, scale line B, or a measurement (by object id, `OBJ-1`) | the line the user clicked |
| cell shape | `Square` or `Rectangle` | `Square` |
| size A | cell size **along** the alignment source | 10 |
| size B | cell size **perpendicular** to the alignment source | 10 |
| unit | the length unit of A and B | the source's unit when the grid was added: the calibration unit for a scale line, or the measurement's own unit for a measurement |
| colour | opaque RGB | green `#00FF00` |
| opacity | 0 to 1 | 0.5 (50 %) |

- **Own unit.** The unit defaults to the source's unit when the grid is added, and from
  then on belongs to the grid. Changing the source's unit later does not change the
  grid. The user can change the grid's unit with the units drop-down in the grid
  properties row (`GRID-14`).
- **Changing the unit keeps the numbers.** Changing a grid's unit keeps the size values
  A and B as they are. For example, 10 cm cells become 10 m cells, not 0.1 m cells.
- **Square cells.** A square cell uses size A for both directions, and size B is kept but
  ignored. Setting the square's side length (L) sets **both** A and B, so switching to
  Rectangle starts from L × L.
- **References, not copies.** The alignment source is a reference. Moving or resizing the
  source line, or re-placing the scale points, moves the grid with it.

**GRID-2 — List.** The document holds an ordered list of grids. New grids are added at the
end. The list is never sorted. Grids are numbered from 1 in list order in the UI.

## Geometry

**GRID-3 — Layout.** The grid is laid out in **real-world space**, then mapped back to the
image:
- **Real-world space.** Image pixels `(x, y)` map to real-world coordinates
  `(x·sx, y·sy)`, using the calibrated scale (`SCALE-17`).
- **Origin and axes.** The origin `O` is the source's start point. `d` is the unit vector
  from the source's start to its end, and `n` is `d` rotated 90°.
- **Spacing.** The cell sizes are converted from the grid's unit to the calibration unit
  (`MEAS-4`), giving spacings `a` (along) and `b` (across), from the shape (`GRID-1`).
- **Two families of lines.** Both pass through `O`:
  - lines **parallel to the source** (direction `d`), at offsets `k·b` along `n`
  - lines **perpendicular to the source** (direction `n`), at offsets `k·a` along `d`

  where `k` is every whole number for which the line meets the document.
- **Clipping.** Each line is clipped to the document rectangle. Lines that only touch a
  corner are dropped. The segments are then mapped back to image pixels.

Laying the grid out in real-world space keeps cells truly square (or rectangular) in the
real world even when `sx ≠ sy`. In image pixels such a cell is a parallelogram.

**GRID-4 — When a grid cannot be drawn.** A grid is not drawn when:
- **Missing source.** Its source no longer exists ("`its alignment line no longer
  exists`"), for example because the measurement was deleted or there are no scale
  points.
- **Zero-length source.** Its source has zero length ("`its alignment line has zero
  length`").
- **No scale.** There is no scale ("`it needs a scale (<scale reason>)`", `SCALE-18`).
- **Too dense.** It would need more than **10 000** lines ("`the cells are too small to
  draw`").

The grid stays in the document. It is drawn again as soon as the condition clears.

## Drawing

**GRID-5 — Appearance.** Grid lines are 1 px wide, in the grid's colour, with the grid's
opacity applied as alpha (`round(opacity × 255)`). They are drawn in **every** tool mode.

**GRID-6 — Drawing order.** The canvas draws, from bottom to top:
1. the image and its outline (`VIEW-6`, `VIEW-7`)
2. **grids**
3. measurement lines (`MEAS-9`)
4. scale lines and markers, when shown (`SCALE-11`, `GRID-8`, `GRID-13`)
5. the grid tool's hover highlight (`GRID-9`)
6. other tool overlays: the rotate arrow and the pending measurement
7. the selection box (`OBJ-8`)

## Grid tool

**GRID-7 — Tool mode.** **Grid** is a tool mode (`SHELL-8`):
- its button comes directly after the Measure distance button (`SHELL-5`)
- its icon means "grid". The current glyph is ⊞ (U+229E, squared plus), from the bundled
  fonts (`SHELL-6a`)
- its tooltip is "`Grid`"

**GRID-8 — Options row.** In Grid mode, with a document open, the tool options row shows:
- **Mode buttons.** Two toggle buttons, "`Via scale`" (**default**) and
  "`Via measurement`", which select the *alignment mode*. Selecting a mode is not an
  action.
- **Hint.** A separator, then a weak hint:

| Mode | Condition | Hint |
|---|---|---|
| Via scale | no scale points | "`Place the scale points first (Scale tool).`" |
| Via scale | otherwise | "`Click a scale line to add a grid aligned to it.`" |
| Via measurement | no measurements | "`Add a measurement first (Measure distance tool).`" |
| Via measurement | otherwise | "`Click a measurement line to add a grid aligned to it.`" |

In Grid mode the canvas shows the document's scale lines and markers (the document's own
point set, whatever method the Scale tool has selected) and all measurement lines.

**GRID-9 — Hover.** In Grid mode, the *candidate lines* depend on the alignment mode:
- **Via scale:** scale lines A and B. There are none without scale points.
- **Via measurement:** every measurement line, using the live position during a resize.

A candidate is **hovered** when the pointer is within **5 screen px** of the line segment.
The segment ends are excluded: a pointer within **8 screen px** of either end of a line
does not count for that line, so handles and markers are never part of the hover area.
- If several lines qualify, the nearest one wins.
- The hovered line is highlighted with a 5 px stroke in translucent yellow
  (`rgba(255, 220, 0, 200)` premultiplied), drawn over it.
- While a line is hovered, the cursor is a pointing hand.

**GRID-10 — Adding a grid.** A primary click whose position hovers a candidate line
(`GRID-9`) commits one **Add grid** action, with a new grid aligned to that line and the
initial properties of `GRID-1`. The new grid then becomes the **active object**.

Clicks that do not hover a candidate add nothing.

## Grids as objects

**GRID-11 — Tree.** The object tree (`OBJ-4`) has a "`Grids`" folder after
"`Measurements`":
- It contains one leaf per grid, in list order (`GRID-2`) and never sorted.
- Each leaf is labelled "`Grid <n> (<source>)`", where `<source>` is one of:
  - "`scale line A`" or "`scale line B`"
  - "`measurement <m>`", where `m` is the measurement's current 1-based position
  - "`alignment line missing`"

Selecting a grid leaf makes it the active object. The view then centres on the midpoint
of its alignment line, if that line exists (`OBJ-5`, `VIEW-17`).

**GRID-12 — Selection.** A grid covers the whole document, so it has **no selection box
and no resize handles** (an exception to `OBJ-8` and `OBJ-9`). While a grid is the active
object, **any** primary click on the canvas clears it (an exception to `OBJ-10`). In Grid
mode, a click that adds a new grid then makes the new grid active.

**GRID-13 — Source always visible.** While a grid is the active object, its alignment
source is always drawn, in every tool mode:
- **Scale-line source:** that scale line, with its label and its two end markers
  (`SCALE-11` style), is drawn even in modes that otherwise hide scale lines.
- **Measurement source:** measurement lines are already drawn in every mode
  (`MEAS-8`).

## Grid properties row

**GRID-14 — Placement.** While the active object is a grid and a document is open, a
*grid properties row* is shown below the tool options row (if any), directly under the
main tool bar area (`SHELL-2`). It wraps onto more lines if there is not enough width.
Groups are separated by vertical separators:

1. "`Grid <n>`", in strong text.
2. "`Cell shape:`", followed by two toggle buttons, "`Square`" and "`Rectangle`".
   Clicking the shape that is not selected commits **Set grid cell shape**.
3. **Cell size:**
   - **Square:** the label "`L`" and one commit-on-blur field (`FMT-9`) for the side
     length.
   - **Rectangle:** the label "`A`" with a field, then "`B`" with a field.
   - Each field is 60 px wide and shows its value formatted by `FMT-7`.
   - Tooltips:
     - L: "`Side length of each square cell`"
     - A: "`Cell size along the alignment line`"
     - B: "`Cell size perpendicular to the alignment line`"
   - A **units drop-down** follows the fields. It shows the grid's unit symbol, lists
     every supported unit (`cm`, `m`), and has the tooltip "`Units of the cell size`".
     Choosing a different unit commits **Set grid units**. Choosing the current unit
     commits nothing.
   - A committed value that parses (`FMT-8`), is greater than 0 and differs from the
     stored value commits **Set grid cell size** for that dimension (L, A or B).
     Anything else reverts.
4. "`Color:`", followed by a 36 × 18 px swatch filled with the grid's colour, with the
   tooltip "`Choose the grid colour`" (`GRID-15`).
5. "`Opacity:`", followed by a slider from 0 to 100 %, showing whole percent
   (`GRID-16`).
6. If the grid cannot be drawn (`GRID-4`): a weak "`Grid not shown: <reason>`".

**GRID-15 — Colour.**
- Clicking the swatch opens a colour picker popup (opaque colours only). It closes when
  the user clicks outside it.
- While the popup is open, the chosen colour is a **preview**. The grid is drawn in it
  live, and the document does not change.
- When the popup closes, one **Set grid colour** action is committed if the colour
  differs from the stored one. Otherwise nothing is committed.

**GRID-16 — Opacity.**
- **Dragging** the slider previews live and commits one **Set grid opacity** action when
  the drag is released.
- A **click** on the slider or a **keyboard** change commits immediately.
- The value is rounded to a whole percent before it is stored.
- Nothing is committed if the value is unchanged.

**GRID-17 — Previews are presentation state.** Previews (`GRID-15`, `GRID-16`) are
discarded whenever the active object changes or the document changes (`OBJ-7`).

## Acceptance criteria

- New grid defaults: square, 10 × 10, the given unit, colour `#00FF00`, opacity 0.5.
- Square cells use A for both dimensions. Rectangle uses A × B. Setting L sets both.
- Geometry, on a 100×50 document at 1 cm/px with the source from (0,0) to the right:
  - 10 cm cells give vertical lines at x = 0, 10, …, 100 (11 lines), each spanning the
    full height, and 6 horizontal lines.
  - Rectangle A = 25, B = 50 gives 5 across-lines and 2 along-lines.
  - A grid aligned to a vertical measurement starting at (10,10) has lines through x = 10
    and y = 10.
  - A 45° source gives lines a whole number of cells from the origin, measured
    perpendicular to each line.
  - With 0.5 cm/px across and 2 cm/px down, 10 cm cells are 20 px wide and 5 px tall.
  - A 0.1 m cell at 1 cm/px is 10 px.
- Errors:
  - No scale gives the no-scale error.
  - A deleted source measurement, or no scale points, gives "source missing".
  - A 0.001 cm cell gives "too dense".
- Actions:
  - **Add grid** gets its own object id.
  - **Add grid** with a missing source is rejected.
  - Shape, size, colour and opacity edits apply.
  - A size of zero or NaN, opacity 1.5, or a missing index is rejected, leaving the
    document unchanged.
  - Setting a grid's unit to m keeps its sizes (10 × 10) and leaves the calibration unit
    unchanged. Setting the unit of a missing grid is rejected.
  - Labels match `HIST-2`.
- Tool:
  - Via scale is the default.
  - Via scale offers both scale lines, and none without scale points.
  - Via measurement offers every measurement.
  - A new grid takes its unit from its source.
- Selection:
  - A grid has no selection box and cannot be resized.
  - A grid preview shows only for the active grid until cleared, and is dropped when the
    selection changes or the document syncs.
- Canvas, simulated with pointer events, in Grid mode via measurement:
  - A click 4 px off the line adds exactly one grid.
  - A click 4 px from the line's end, or 7 px off the line, adds nothing.
  - Hovering highlights the line, and moving away clears the highlight.
  - Via scale ignores measurement lines.
- Canvas: any click clears an active grid.
- Hit-testing: perpendicular distance inside the segment, end distance outside it.
  Points near the ends are excluded. The nearest of two crossing lines wins.
- In the tool bar, Grid comes directly after Measure distance. The grid icon is a single
  glyph present in the bundled fonts.
